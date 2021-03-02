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

    mod string {
        use super::*;

        mod parse {
            use super::*;

            #[test]
            fn empty() {
                assert_eq!(
                    parse(""),
                    Err(Error {
                        kind: ErrorKind::ExpectedNumber,
                        range: 0..0
                    })
                );
            }

            #[test]
            fn invalid_number() {
                assert_eq!(
                    parse("x"),
                    Err(Error {
                        kind: ErrorKind::ExpectedNumber,
                        range: 0..1
                    })
                );
            }

            #[test]
            fn invalid_index() {
                assert_eq!(
                    parse("0"),
                    Err(Error {
                        kind: ErrorKind::IndexZero,
                        range: 0..1
                    })
                );
            }

            #[test]
            fn valid_index() {
                assert_eq!(parse("1"), Ok(make_column(0, "\t")));
            }

            #[test]
            fn missing_separator() {
                assert_eq!(
                    parse("1:"),
                    Err(Error {
                        kind: ErrorKind::ExpectedColumnSeparator,
                        range: 2..2
                    })
                );
            }

            #[test]
            fn with_separator() {
                assert_eq!(parse("10:abc"), Ok(make_column(9, "abc")));
            }

            fn parse(value: &str) -> Result<Column> {
                Column::parse(&mut Reader::from(value), &make_separator("\t"))
            }
        }

        mod get {
            use super::*;

            #[test]
            fn empty_separator() {
                assert_eq!(make_column(0, "").get("a b c"), "")
            }

            #[test]
            fn first() {
                assert_eq!(make_column(0, " ").get("a b c"), "a")
            }

            #[test]
            fn first_when_first_empty() {
                assert_eq!(make_column(0, " ").get(" a b"), "")
            }

            #[test]
            fn first_when_last_empty() {
                assert_eq!(make_column(0, " ").get("a b "), "a")
            }

            #[test]
            fn last() {
                assert_eq!(make_column(2, " ").get("a b c"), "c")
            }

            #[test]
            fn last_when_first_empty() {
                assert_eq!(make_column(2, " ").get(" a b"), "b")
            }

            #[test]
            fn last_when_last_empty() {
                assert_eq!(make_column(2, " ").get("a b "), "")
            }

            #[test]
            fn over_last() {
                assert_eq!(make_column(2, " ").get("a b"), "")
            }
        }

        mod get_backward {
            use super::*;

            #[test]
            fn empty_separator() {
                assert_eq!(make_column(0, "").get_backward("a b c"), "")
            }

            #[test]
            fn first() {
                assert_eq!(make_column(0, " ").get_backward("a b c"), "c")
            }

            #[test]
            fn first_when_first_empty() {
                assert_eq!(make_column(0, " ").get_backward("a b "), "")
            }

            #[test]
            fn first_when_last_empty() {
                assert_eq!(make_column(0, " ").get_backward(" a b"), "b")
            }

            #[test]
            fn last() {
                assert_eq!(make_column(2, " ").get_backward("a b c"), "a")
            }

            #[test]
            fn last_when_first_empty() {
                assert_eq!(make_column(2, " ").get_backward("a b "), "a")
            }

            #[test]
            fn last_when_last_empty() {
                assert_eq!(make_column(2, " ").get_backward(" a b"), "")
            }

            #[test]
            fn over_last() {
                assert_eq!(make_column(2, " ").get_backward("a b"), "")
            }
        }

        #[test]
        fn display() {
            assert_eq!(make_column(1, "_").to_string(), "column #2 ('_' separator)");
        }

        fn make_column(index: usize, separator: &str) -> Column {
            Column {
                index,
                separator: make_separator(separator),
            }
        }

        fn make_separator(value: &str) -> Separator {
            Separator::String(String::from(value))
        }
    }

    mod regex {
        extern crate regex;
        use super::*;
        use regex::Regex;

        mod parse {
            use super::*;
            use crate::utils::AnyString;

            #[test]
            fn valid() {
                assert_eq!(parse("2/[0-9]+"), Ok(make_column(1, "[0-9]+")));
            }

            #[test]
            fn invalid() {
                assert_eq!(
                    parse("2/[0-9"),
                    Err(Error {
                        kind: ErrorKind::RegexInvalid(AnyString(String::from(
                            "This string is not compared by assertion"
                        ))),
                        range: 2..6
                    })
                );
            }

            fn parse(value: &str) -> Result<Column> {
                Column::parse(&mut Reader::from(value), &make_separator("\\s+"))
            }
        }

        mod get {
            use super::*;

            #[test]
            fn empty_separator() {
                assert_eq!(make_column(0, "").get("a\t\tb\t\tc"), "")
            }

            #[test]
            fn first() {
                assert_eq!(make_column(0, "\\s+").get("a\t\tb\t\tc"), "a")
            }

            #[test]
            fn first_when_first_empty() {
                assert_eq!(make_column(0, "\\s+").get("\t\ta\t\tb"), "")
            }

            #[test]
            fn first_when_last_empty() {
                assert_eq!(make_column(0, "\\s+").get("a\t\tb\t\t"), "a")
            }

            #[test]
            fn last() {
                assert_eq!(make_column(2, "\\s+").get("a\t\tb\t\tc"), "c")
            }

            #[test]
            fn last_when_first_empty() {
                assert_eq!(make_column(2, "\\s+").get("\t\ta\t\tb"), "b")
            }

            #[test]
            fn last_when_last_empty() {
                assert_eq!(make_column(2, "\\s+").get("a\t\tb\t\t"), "")
            }

            #[test]
            fn over_last() {
                assert_eq!(make_column(2, "\\s+").get("a\t\tb"), "")
            }
        }

        mod get_backward {
            use super::*;

            #[test]
            fn empty_separator() {
                assert_eq!(make_column(0, "").get_backward("a\t\tb\t\tc"), "")
            }

            #[test]
            fn first() {
                assert_eq!(make_column(0, "\\s+").get_backward("a\t\tb\t\tc"), "c")
            }

            #[test]
            fn first_when_first_empty() {
                assert_eq!(make_column(0, "\\s+").get_backward("a\t\tb\t\t"), "")
            }

            #[test]
            fn first_when_last_empty() {
                assert_eq!(make_column(0, "\\s+").get_backward("\t\ta\t\tb"), "b")
            }

            #[test]
            fn last() {
                assert_eq!(make_column(2, "\\s+").get_backward("a\t\tb\t\tc"), "a")
            }

            #[test]
            fn last_when_first_empty() {
                assert_eq!(make_column(2, "\\s+").get_backward("a\t\tb\t\t"), "a")
            }

            #[test]
            fn last_when_last_empty() {
                assert_eq!(make_column(2, "\\s+").get_backward("\t\ta\t\tb"), "")
            }

            #[test]
            fn over_last() {
                assert_eq!(make_column(2, "\\s+").get_backward("a\t\tb"), "")
            }
        }

        #[test]
        fn display() {
            assert_eq!(
                make_column(2, "[0-9]+").to_string(),
                "column #3 (regular expression '[0-9]+' separator)"
            );
        }

        fn make_column(index: usize, separator: &str) -> Column {
            Column {
                index,
                separator: make_separator(separator),
            }
        }

        fn make_separator(value: &str) -> Separator {
            Separator::Regex(RegexHolder(Regex::new(value).unwrap()))
        }
    }
}
