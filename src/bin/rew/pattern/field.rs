use crate::pattern::char::Char;
use crate::pattern::index::parse_index;
use crate::pattern::parse::{Error, ErrorKind, Result, Separator};
use crate::pattern::reader::Reader;
use std::convert::TryInto;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Field {
    pub index: usize,
    pub separator: Separator,
}

impl Field {
    pub fn parse(reader: &mut Reader<Char>, default_separator: &Separator) -> Result<Self> {
        let index = parse_index(reader)?;

        if let Some(delimiter) = reader.read_char() {
            let separator_start = reader.position();
            let separator = reader.read_to_end().to_string();

            if separator.is_empty() {
                return Err(Error {
                    kind: ErrorKind::ExpectedFieldSeparator,
                    range: separator_start..reader.position(),
                });
            }

            let separator = if delimiter == '/' {
                match separator.try_into() {
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

    pub fn get<'a>(&self, value: &'a str) -> &'a str {
        match &self.separator {
            Separator::String(separator) => value.split(separator).nth(self.index).unwrap_or(""),
            Separator::Regex(separator) => separator.split(value).nth(self.index).unwrap_or(""),
        }
    }

    pub fn get_rev<'a>(&self, value: &'a str) -> &'a str {
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

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "field #{} ({} separator)",
            self.index + 1,
            self.separator
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{AnyString, IndexRange};
    use test_case::test_case;

    #[test_case("",     0..0, ErrorKind::ExpectedNumber                 ; "empty")]
    #[test_case("x",    0..1, ErrorKind::ExpectedNumber                 ; "invalid number")]
    #[test_case("0",    0..1, ErrorKind::IndexZero                      ; "invalid index")]
    #[test_case("1:",   2..2, ErrorKind::ExpectedFieldSeparator         ; "missing string separator")]
    #[test_case("1/",   2..2, ErrorKind::ExpectedFieldSeparator         ; "missing regex separator")]
    #[test_case("1/[0", 2..4, ErrorKind::RegexInvalid(AnyString::any()) ; "invalid regex separator")]
    fn parse_err(input: &str, range: IndexRange, kind: ErrorKind) {
        assert_eq!(
            Field::parse(&mut Reader::from(input), &Separator::String(' '.into())),
            Err(Error { kind, range })
        );
    }

    mod string {
        use super::*;
        use test_case::test_case;

        #[test_case("1",         0, "default" ; "index")]
        #[test_case("10:custom", 9, "custom"  ; "index and separator")]
        fn parse(input: &str, index: usize, separator: &str) {
            assert_eq!(
                Field::parse(
                    &mut Reader::from(input),
                    &Separator::String("default".into())
                ),
                Ok(Field {
                    index,
                    separator: Separator::String(separator.into())
                })
            );
        }

        #[test_case("a b c", 0, "",  ""  ; "empty separator")]
        #[test_case("a b c", 0, " ", "a" ; "first")]
        #[test_case(" a b",  0, " ", ""  ; "first when first empty")]
        #[test_case("a b ",  0, " ", "a" ; "first when last empty")]
        #[test_case("a b c", 2, " ", "c" ; "last")]
        #[test_case(" a b",  2, " ", "b" ; "last when first empty")]
        #[test_case("a b ",  2, " ", ""  ; "last when last empty")]
        #[test_case("a b",   2, " ", ""  ; "over last")]
        fn get(input: &str, index: usize, separator: &str, output: &str) {
            assert_eq!(
                Field {
                    index,
                    separator: Separator::String(separator.into())
                }
                .get(input),
                output
            );
        }

        #[test_case("a b c", 0, "",  ""  ; "empty separator")]
        #[test_case("a b c", 0, " ", "c" ; "first")]
        #[test_case("a b ",  0, " ", ""  ; "first when first empty")]
        #[test_case(" a b",  0, " ", "b" ; "first when last empty")]
        #[test_case("a b c", 2, " ", "a" ; "last")]
        #[test_case("a b ",  2, " ", "a" ; "last when first empty")]
        #[test_case(" a b",  2, " ", ""  ; "last when last empty")]
        #[test_case("a b",   2, " ", ""  ; "over last")]
        fn get_rev(input: &str, index: usize, separator: &str, output: &str) {
            assert_eq!(
                Field {
                    index,
                    separator: Separator::String(separator.into())
                }
                .get_rev(input),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                Field {
                    index: 1,
                    separator: Separator::String("_".into())
                }
                .to_string(),
                "field #2 ('_' separator)"
            );
        }
    }

    mod regex {
        extern crate regex;
        use super::*;
        use test_case::test_case;

        #[test_case("1",         0, "\\s+"   ; "index")]
        #[test_case("10/[0-9]+", 9, "[0-9]+" ; "index and separator")]
        fn parse(input: &str, index: usize, separator: &str) {
            assert_eq!(
                Field::parse(&mut Reader::from(input), &Separator::Regex("\\s+".into())),
                Ok(Field {
                    index,
                    separator: Separator::Regex(separator.into())
                })
            );
        }

        #[test_case("a\t\tb\t\tc", 0, "",     ""  ; "empty separator")]
        #[test_case("a\t\tb\t\tc", 0, "\\s+", "a" ; "first")]
        #[test_case("\t\ta\t\tb",  0, "\\s+", ""  ; "first when first empty")]
        #[test_case("a\t\tb\t\t",  0, "\\s+", "a" ; "first when last empty")]
        #[test_case("a\t\tb\t\tc", 2, "\\s+", "c" ; "last")]
        #[test_case("\t\ta\t\tb",  2, "\\s+", "b" ; "last when first empty")]
        #[test_case("a\t\tb\t\t",  2, "\\s+", ""  ; "last when last empty")]
        #[test_case("a\t\tb",      2, "\\s+", ""  ; "over last")]
        fn get(input: &str, index: usize, separator: &str, output: &str) {
            assert_eq!(
                Field {
                    index,
                    separator: Separator::Regex(separator.into())
                }
                .get(input),
                output
            );
        }

        #[test_case("a\t\tb\t\tc", 0, "",     ""  ; "empty separator")]
        #[test_case("a\t\tb\t\tc", 0, "\\s+", "c" ; "first")]
        #[test_case("a\t\tb\t\t",  0, "\\s+", ""  ; "first when first empty")]
        #[test_case("\t\ta\t\tb",  0, "\\s+", "b" ; "first when last empty")]
        #[test_case("a\t\tb\t\tc", 2, "\\s+", "a" ; "last")]
        #[test_case("a\t\tb\t\t",  2, "\\s+", "a" ; "last when first empty")]
        #[test_case("\t\ta\t\tb",  2, "\\s+", ""  ; "last when last empty")]
        #[test_case("a\t\tb",      2, "\\s+", ""  ; "over last")]
        fn get_rev(input: &str, index: usize, separator: &str, output: &str) {
            assert_eq!(
                Field {
                    index,
                    separator: Separator::Regex(separator.into())
                }
                .get_rev(input),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                Field {
                    index: 1,
                    separator: Separator::Regex("[0-9]+".into())
                }
                .to_string(),
                "field #2 (regular expression '[0-9]+' separator)"
            );
        }
    }
}
