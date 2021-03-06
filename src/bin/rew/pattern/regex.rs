use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::Deref;

use lazy_static::lazy_static;
use regex::Regex;

use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::utils::AnyString;

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
            value.to_string().try_into().map_err(|kind| Error {
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

#[cfg(test)]
impl From<&str> for RegexHolder {
    fn from(value: &str) -> Self {
        Self(Regex::new(value).unwrap())
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
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("",            ""                  ; "empty")]
    #[test_case("ab",          "ab"                ; "zero")]
    #[test_case("a$1b",        "a${1}b"            ; "one")]
    #[test_case("$1a$12b$123", "${1}a${12}b${123}" ; "multiple")]
    fn add_capture_group_brackets(input: &str, output: &str) {
        assert_eq!(super::add_capture_group_brackets(input), output)
    }

    mod regex_holder {
        use test_case::test_case;

        use super::*;

        mod try_from {
            use test_case::test_case;

            use super::*;

            #[test]
            fn err() {
                assert_eq!(
                    RegexHolder::try_from(String::from("[0-9")),
                    Err(ErrorKind::RegexInvalid(AnyString::any()))
                );
            }

            #[test_case("",       ""       ; "empty")]
            #[test_case("[a-z]+", "[a-z]+" ; "noempty")]
            fn ok(input: &str, output: &str) {
                assert_eq!(
                    RegexHolder::try_from(String::from(input)),
                    Ok(output.into())
                );
            }
        }

        mod parse {
            use test_case::test_case;

            use super::*;
            use crate::pattern::error::ErrorRange;

            #[test_case("",     0..0, ErrorKind::ExpectedRegex                  ; "empty")]
            #[test_case("[0-9", 0..4, ErrorKind::RegexInvalid(AnyString::any()) ; "invalid")]
            fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
                assert_eq!(
                    RegexHolder::parse(&mut Reader::from(input)),
                    Err(Error { kind, range })
                );
            }

            #[test]
            fn ok() {
                assert_eq!(
                    RegexHolder::parse(&mut Reader::from("[0-9]")),
                    Ok("[0-9]".into())
                );
            }
        }

        #[test_case("",             "\\d+", ""    ; "empty")]
        #[test_case("abc",          "\\d+", ""    ; "none")]
        #[test_case("abc123def456", "\\d+", "123" ; "first")]
        fn first_match(input: &str, regex: &str, output: &str) {
            assert_eq!(RegexHolder::from(regex).first_match(input), output);
        }

        #[test_case("",       "",       true  ; "empty")]
        #[test_case("[a-z]+", "[a-z]+", true  ; "same")]
        #[test_case("[a-z]+", "[a-z]*", false ; "different")]
        fn partial_eq(left: &str, right: &str, result: bool) {
            assert_eq!(RegexHolder::from(left) == RegexHolder::from(right), result);
        }

        #[test]
        fn display() {
            assert_eq!(RegexHolder::from("[a-z]+").to_string(), "[a-z]+");
        }
    }
}
