use crate::pattern::char::{AsChar, Char};
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use regex::Regex;
use std::fmt;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct Substitution<T> {
    pub value: T,
    pub replacement: String,
}

impl Substitution<String> {
    pub fn parse_string(reader: &mut Reader<Char>) -> Result<Self> {
        let (value, _, replacement) = parse_value_and_replacement(reader)?;
        Ok(Self { value, replacement })
    }
}

impl Substitution<RegexHolder> {
    pub fn parse_regex(reader: &mut Reader<Char>) -> Result<Self> {
        let (value, value_range, replacement) = parse_value_and_replacement(reader)?;
        let value = match Regex::new(&value) {
            Ok(regex) => RegexHolder(regex),
            Err(error) => {
                return Err(Error {
                    kind: ErrorKind::SubstituteRegexInvalid(error.to_string()),
                    range: value_range,
                })
            }
        };
        Ok(Self { value, replacement })
    }
}

pub fn parse_value_and_replacement(
    reader: &mut Reader<Char>,
) -> Result<(String, Range<usize>, String)> {
    if let Some(separator) = reader.read().cloned() {
        let mut value = String::new();
        let value_start = reader.position();
        let mut value_end = value_start;

        while let Some(ch) = reader.read_char() {
            if ch == separator.as_char() {
                break;
            } else {
                value.push(ch);
                value_end = reader.position();
            }
        }

        if value.is_empty() {
            return Err(Error {
                kind: ErrorKind::SubstituteWithoutValue(separator),
                range: value_start..value_end,
            });
        }

        Ok((
            value,
            value_start..value_end,
            Char::join(reader.read_to_end()),
        ))
    } else {
        Err(Error {
            kind: ErrorKind::ExpectedSubstitution,
            range: reader.position()..reader.end(),
        })
    }
}

impl<T: fmt::Display> fmt::Display for Substitution<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "'{}' by '{}'", self.value, self.replacement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_error() {
        let mut reader = Reader::from("");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Err(Error {
                kind: ErrorKind::ExpectedSubstitution,
                range: 0..0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn no_value_error() {
        let mut reader = Reader::from("/");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Err(Error {
                kind: ErrorKind::SubstituteWithoutValue(Char::Raw('/')),
                range: 1..1,
            })
        );
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn empty_value_error() {
        let mut reader = Reader::from("//");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Err(Error {
                kind: ErrorKind::SubstituteWithoutValue(Char::Raw('/')),
                range: 1..1,
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn value_no_replacement() {
        let mut reader = Reader::from("/a");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("a"), 1..2, String::from("")))
        );
        assert_eq!(reader.position(), 2)
    }

    #[test]
    fn long_value_no_replacement() {
        let mut reader = Reader::from("/abc");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("abc"), 1..4, String::from("")))
        );
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn value_empty_replacement() {
        let mut reader = Reader::from("/a/");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("a"), 1..2, String::from("")))
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn long_value_empty_replacement() {
        let mut reader = Reader::from("/abc/");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("abc"), 1..4, String::from("")))
        );
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn value_replacement() {
        let mut reader = Reader::from("/a/d");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("a"), 1..2, String::from("d")))
        );
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn long_value_replacement() {
        let mut reader = Reader::from("/abc/def");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("abc"), 1..4, String::from("def")))
        );
        assert_eq!(reader.position(), 8);
    }

    #[test]
    fn value_replacement_with_redundant_separators() {
        let mut reader = Reader::from("/abc/d//e/");
        assert_eq!(
            parse_value_and_replacement(&mut reader),
            Ok((String::from("abc"), 1..4, String::from("d//e/")))
        );
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn parse_string() {
        let mut reader = Reader::from("/abc/def");
        assert_eq!(
            Substitution::parse_string(&mut reader),
            Ok(Substitution {
                value: String::from("abc"),
                replacement: String::from("def"),
            })
        );
        assert_eq!(reader.position(), 8);
    }

    #[test]
    fn parse_regex() {
        let mut reader = Reader::from("/[0-9]+/def");
        assert_eq!(
            Substitution::parse_regex(&mut reader),
            Ok(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::from("def"),
            })
        );
        assert_eq!(reader.position(), 11);
    }

    #[test]
    fn parse_regex_error() {
        let mut reader = Reader::from("/[0-9+/def");
        assert_eq!(
            Substitution::parse_regex(&mut reader),
            Err(Error {
                kind: ErrorKind::SubstituteRegexInvalid(String::from(
                    "regex parse error:
    [0-9+
    ^
error: unclosed character class"
                )),
                range: 1..6,
            })
        );
        assert_eq!(reader.position(), 10);
    }
}
