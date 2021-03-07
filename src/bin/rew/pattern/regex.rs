use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::utils::AnyString;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$(\d+)").unwrap();
}

pub fn add_capture_group_brackets(string: &str) -> Cow<str> {
    if string.contains('$') {
        CAPTURE_GROUP_VAR_REGEX.replace_all(string, r"$${${1}}")
    } else {
        Cow::Borrowed(string)
    }
}

#[derive(Debug, Clone)]
pub struct RegexHolder(pub Regex);

impl RegexHolder {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        let value_start = reader.position();
        let value = reader.read_to_end();

        if value.is_empty() {
            Err(Error {
                kind: ErrorKind::ExpectedRegex,
                range: value_start..value_start,
            })
        } else {
            Self::try_from(value.to_string()).map_err(|kind| Error {
                kind,
                range: value_start..reader.position(),
            })
        }
    }

    pub fn first_match(&self, value: &str) -> String {
        match self.0.find(value) {
            Some(result) => result.as_str().to_string(),
            None => String::new(),
        }
    }
}

impl TryFrom<String> for RegexHolder {
    type Error = ErrorKind;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match Regex::new(&value) {
            Ok(regex) => Ok(Self(regex)),
            Err(error) => Err(ErrorKind::RegexInvalid(AnyString(error.to_string()))),
        }
    }
}

impl Deref for RegexHolder {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for RegexHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl fmt::Display for RegexHolder {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
impl From<&str> for RegexHolder {
    fn from(value: &str) -> Self {
        Self(Regex::new(value).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod add_capture_group_brackets {
        use super::*;

        #[test]
        fn zero() {
            assert_eq!(add_capture_group_brackets("ab"), "ab");
        }

        #[test]
        fn one() {
            assert_eq!(add_capture_group_brackets("a$1b"), "a${1}b");
        }

        #[test]
        fn multiple() {
            assert_eq!(
                add_capture_group_brackets("$1a$12b$123"),
                "${1}a${12}b${123}"
            );
        }
    }

    mod regex_holder {
        use super::*;

        mod try_from {
            use super::*;

            #[test]
            fn valid() {
                assert_eq!(
                    RegexHolder::try_from(String::from("[0-9]")),
                    Ok(RegexHolder::from("[0-9]"))
                );
            }

            #[test]
            fn invalid() {
                assert_eq!(
                    RegexHolder::try_from(String::from("[0-9")),
                    Err(ErrorKind::RegexInvalid(AnyString::any()))
                );
            }
        }

        mod parse {
            use super::*;

            #[test]
            fn empty() {
                let mut reader = Reader::from("");
                assert_eq!(
                    RegexHolder::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRegex,
                        range: 0..0,
                    })
                );
                assert_eq!(reader.position(), 0);
            }

            #[test]
            fn valid() {
                let mut reader = Reader::from("[0-9]");
                assert_eq!(
                    RegexHolder::parse(&mut reader),
                    Ok(RegexHolder::from("[0-9]"))
                );
                assert_eq!(reader.position(), 5);
            }

            #[test]
            fn invalid() {
                let mut reader = Reader::from("[0-9");
                assert_eq!(
                    RegexHolder::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RegexInvalid(AnyString::any()),
                        range: 0..4,
                    })
                );
                assert_eq!(reader.position(), 4);
            }
        }

        mod first_match {
            use super::*;

            #[test]
            fn empty() {
                assert_eq!(RegexHolder::from("\\d+").first_match(""), String::new());
            }

            #[test]
            fn none() {
                assert_eq!(RegexHolder::from("\\d+").first_match("abc"), String::new());
            }

            #[test]
            fn first() {
                assert_eq!(
                    RegexHolder::from("\\d+").first_match("abc123def456"),
                    String::from("123")
                );
            }
        }

        #[test]
        fn partial_eq() {
            assert_eq!(RegexHolder::from("[a-z]+"), RegexHolder::from("[a-z]+"));
            assert_ne!(RegexHolder::from("[a-z]+"), RegexHolder::from("[a-z]*"));
        }

        #[test]
        fn display() {
            assert_eq!(
                RegexHolder::from("[a-z]+").to_string(),
                String::from("[a-z]+")
            );
        }
    }
}
