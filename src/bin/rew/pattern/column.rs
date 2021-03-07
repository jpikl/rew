use crate::pattern::char::Char;
use crate::pattern::index::{Index, IndexValue};
use crate::pattern::parse::{Error, ErrorKind, Result, Separator};
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Column {
    pub index: IndexValue,
    pub separator: Separator,
}

impl Column {
    pub fn parse(reader: &mut Reader<Char>, default_separator: &Separator) -> Result<Self> {
        let index = Index::parse(reader)?;

        if let Some(delimiter) = reader.read_char() {
            let separator_start = reader.position();
            let separator = reader.read_to_end().to_string();

            if separator.is_empty() {
                return Err(Error {
                    kind: ErrorKind::ExpectedColumnSeparator,
                    range: separator_start..reader.position(),
                });
            }

            let separator = if delimiter == '/' {
                match RegexHolder::try_from(separator) {
                    Ok(regex) => Separator::Regex(regex),
                    Err(kind) => {
                        return Err(Error {
                            kind,
                            range: separator_start..reader.position(),
                        })
                    }
                }
            } else {
                Separator::String(separator)
            };
            Ok(Self { index, separator })
        } else {
            Ok(Self {
                index,
                separator: default_separator.clone(),
            })
        }
    }
}

impl Column {
    pub fn get<'a>(&self, value: &'a str) -> &'a str {
        match &self.separator {
            Separator::String(separator) => value.split(separator).nth(self.index).unwrap_or(""),
            Separator::Regex(separator) => separator.split(value).nth(self.index).unwrap_or(""),
        }
    }

    pub fn get_backward<'a>(&self, value: &'a str) -> &'a str {
        match &self.separator {
            Separator::String(separator) => value.rsplit(separator).nth(self.index).unwrap_or(""),
            Separator::Regex(separator) => {
                // Regex does not support rsplit or DoubleEndedIterator
                let count = separator.split(value).count();
                if self.index < count {
                    let index = count - self.index - 1;
                    separator.split(value).nth(index).unwrap_or("")
                } else {
                    ""
                }
            }
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "column #{} ({} separator)",
            self.index + 1,
            self.separator
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;

    mod string {
        use super::*;
        use test_case::test_case;

        #[test_case("", ErrorKind::ExpectedNumber, 0..0; "empty")]
        #[test_case("x", ErrorKind::ExpectedNumber, 0..1; "invalid number")]
        #[test_case("0", ErrorKind::IndexZero, 0..1; "invalid index")]
        #[test_case("1:", ErrorKind::ExpectedColumnSeparator, 2..2; "missing separator")]
        fn parse_err(input: &str, kind: ErrorKind, range: Range<usize>) {
            assert_eq!(
                Column::parse(
                    &mut Reader::from(input),
                    &Separator::String(String::from('\t'))
                ),
                Err(Error { kind, range })
            );
        }

        #[test_case("1", 0, "\t"; "index")]
        #[test_case("10:abc", 9, "abc"; "index and separator")]
        fn parse_ok(input: &str, index: IndexValue, separator: &str) {
            assert_eq!(
                Column::parse(
                    &mut Reader::from(input),
                    &Separator::String(String::from('\t'))
                ),
                Ok(Column {
                    index,
                    separator: Separator::String(String::from(separator))
                })
            );
        }

        #[test_case(0, "",  "a b c", ""; "empty separator")]
        #[test_case(0, " ", "a b c", "a"; "first")]
        #[test_case(0, " ", " a b", ""; "first when first empty")]
        #[test_case(0, " ", "a b ", "a"; "first when last empty")]
        #[test_case(2, " ", "a b c", "c"; "last")]
        #[test_case(2, " ", " a b", "b"; "last when first empty")]
        #[test_case(2, " ", "a b ", ""; "last when last empty")]
        #[test_case(2, " ", "a b", ""; "over last")]
        fn get(index: IndexValue, separator: &str, input: &str, output: &str) {
            assert_eq!(
                Column {
                    index,
                    separator: Separator::String(String::from(separator))
                }
                .get(input),
                output
            );
        }

        #[test_case(0, "",  "a b c", ""; "empty separator")]
        #[test_case(0, " ", "a b c", "c"; "first")]
        #[test_case(0, " ", "a b ", ""; "first when first empty")]
        #[test_case(0, " ", " a b", "b"; "first when last empty")]
        #[test_case(2, " ", "a b c", "a"; "last")]
        #[test_case(2, " ", "a b ", "a"; "last when first empty")]
        #[test_case(2, " ", " a b", ""; "last when last empty")]
        #[test_case(2, " ", "a b", ""; "over last")]
        fn get_backward(index: IndexValue, separator: &str, input: &str, output: &str) {
            assert_eq!(
                Column {
                    index,
                    separator: Separator::String(String::from(separator))
                }
                .get_backward(input),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                Column {
                    index: 1,
                    separator: Separator::String(String::from("_"))
                }
                .to_string(),
                "column #2 ('_' separator)"
            );
        }
    }

    mod regex {
        extern crate regex;
        use super::*;
        use crate::utils::AnyString;
        use test_case::test_case;

        #[test_case("1/", ErrorKind::ExpectedColumnSeparator, 2..2; "missing separator")]
        #[test_case("1/[0-9", ErrorKind::RegexInvalid(AnyString::any()), 2..6; "invalid regex")]
        fn parse_err(input: &str, kind: ErrorKind, range: Range<usize>) {
            assert_eq!(
                Column::parse(
                    &mut Reader::from(input),
                    &Separator::Regex(RegexHolder::from("\\s+"))
                ),
                Err(Error { kind, range })
            );
        }

        #[test_case("1", 0, "\\s+"; "index")]
        #[test_case("10/[0-9]+", 9, "[0-9]+"; "index and separator")]
        fn parse_ok(input: &str, index: IndexValue, separator: &str) {
            assert_eq!(
                Column::parse(
                    &mut Reader::from(input),
                    &Separator::Regex(RegexHolder::from("\\s+"))
                ),
                Ok(Column {
                    index,
                    separator: Separator::Regex(RegexHolder::from(separator))
                })
            );
        }

        #[test_case(0, "", "a\t\tb\t\tc", ""; "empty separator")]
        #[test_case(0, "\\s+", "a\t\tb\t\tc", "a"; "first")]
        #[test_case(0, "\\s+", "\t\ta\t\tb", ""; "first when first empty")]
        #[test_case(0, "\\s+", "a\t\tb\t\t", "a"; "first when last empty")]
        #[test_case(2, "\\s+", "a\t\tb\t\tc", "c"; "last")]
        #[test_case(2, "\\s+", "\t\ta\t\tb", "b"; "last when first empty")]
        #[test_case(2, "\\s+", "a\t\tb\t\t", ""; "last when last empty")]
        #[test_case(2, "\\s+", "a\t\tb", ""; "over last")]
        fn get(index: IndexValue, separator: &str, input: &str, output: &str) {
            assert_eq!(
                Column {
                    index,
                    separator: Separator::Regex(RegexHolder::from(separator))
                }
                .get(input),
                output
            );
        }

        #[test_case(0, "",  "a\t\tb\t\tc", ""; "empty separator")]
        #[test_case(0, "\\s+", "a\t\tb\t\tc", "c"; "first")]
        #[test_case(0, "\\s+", "a\t\tb\t\t", ""; "first when first empty")]
        #[test_case(0, "\\s+", "\t\ta\t\tb", "b"; "first when last empty")]
        #[test_case(2, "\\s+", "a\t\tb\t\tc", "a"; "last")]
        #[test_case(2, "\\s+", "a\t\tb\t\t", "a"; "last when first empty")]
        #[test_case(2, "\\s+", "\t\ta\t\tb", ""; "last when last empty")]
        #[test_case(2, "\\s+", "a\t\tb", ""; "over last")]
        fn get_backward(index: IndexValue, separator: &str, input: &str, output: &str) {
            assert_eq!(
                Column {
                    index,
                    separator: Separator::Regex(RegexHolder::from(separator))
                }
                .get_backward(input),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                Column {
                    index: 1,
                    separator: Separator::Regex(RegexHolder::from("[0-9]+"))
                }
                .to_string(),
                "column #2 (regular expression '[0-9]+' separator)"
            );
        }
    }
}
