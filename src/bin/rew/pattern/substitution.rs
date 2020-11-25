use crate::pattern::char::{AsChar, Char};
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use crate::utils::AnyString;
use regex::Regex;
use std::fmt;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct Substitution<T> {
    pub target: T,
    pub replacement: String,
}

impl Substitution<String> {
    pub fn parse_string(reader: &mut Reader<Char>) -> Result<Self> {
        let (target, _, replacement) = parse_target_and_replacement(reader)?;
        Ok(Self {
            target,
            replacement,
        })
    }
}

impl Substitution<RegexHolder> {
    pub fn parse_regex(reader: &mut Reader<Char>) -> Result<Self> {
        let (target, target_range, replacement) = parse_target_and_replacement(reader)?;
        let target = match Regex::new(&target) {
            Ok(regex) => RegexHolder(regex),
            Err(error) => {
                return Err(Error {
                    kind: ErrorKind::SubstitutionRegexInvalid(AnyString(error.to_string())),
                    range: target_range,
                })
            }
        };
        Ok(Self {
            target,
            replacement,
        })
    }
}

pub fn parse_target_and_replacement(
    reader: &mut Reader<Char>,
) -> Result<(String, Range<usize>, String)> {
    if let Some(delimiter) = reader.read().cloned() {
        let mut target = String::new();
        let target_start = reader.position();
        let mut target_end = target_start;

        while let Some(ch) = reader.read_char() {
            if ch == delimiter.as_char() {
                break;
            } else {
                target.push(ch);
                target_end = reader.position();
            }
        }

        if target.is_empty() {
            return Err(Error {
                kind: ErrorKind::SubstitutionWithoutTarget(delimiter),
                range: target_start..target_end,
            });
        }

        Ok((
            target,
            target_start..target_end,
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
        write!(formatter, "'{}' with '{}'", self.target, self.replacement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_target_and_replacement {
        use super::*;

        #[test]
        fn empty() {
            let mut reader = Reader::from("");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Err(Error {
                    kind: ErrorKind::ExpectedSubstitution,
                    range: 0..0,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn no_target() {
            let mut reader = Reader::from("/");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Err(Error {
                    kind: ErrorKind::SubstitutionWithoutTarget(Char::Raw('/')),
                    range: 1..1,
                })
            );
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn empty_target() {
            let mut reader = Reader::from("//");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Err(Error {
                    kind: ErrorKind::SubstitutionWithoutTarget(Char::Raw('/')),
                    range: 1..1,
                })
            );
            assert_eq!(reader.position(), 2);
        }

        #[test]
        fn short_target_no_replacement() {
            let mut reader = Reader::from("/a");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("a"), 1..2, String::new()))
            );
            assert_eq!(reader.position(), 2)
        }

        #[test]
        fn long_target_no_replacement() {
            let mut reader = Reader::from("/abc");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("abc"), 1..4, String::new()))
            );
            assert_eq!(reader.position(), 4);
        }

        #[test]
        fn short_target_empty_replacement() {
            let mut reader = Reader::from("/a/");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("a"), 1..2, String::new()))
            );
            assert_eq!(reader.position(), 3);
        }

        #[test]
        fn long_target_empty_replacement() {
            let mut reader = Reader::from("/abc/");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("abc"), 1..4, String::new()))
            );
            assert_eq!(reader.position(), 5);
        }

        #[test]
        fn short_target_short_replacement() {
            let mut reader = Reader::from("/a/d");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("a"), 1..2, String::from("d")))
            );
            assert_eq!(reader.position(), 4);
        }

        #[test]
        fn long_target_long_replacement() {
            let mut reader = Reader::from("/abc/def");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("abc"), 1..4, String::from("def")))
            );
            assert_eq!(reader.position(), 8);
        }

        #[test]
        fn long_target_long_replacement_containing_delimiters() {
            let mut reader = Reader::from("/abc/d//e/");
            assert_eq!(
                parse_target_and_replacement(&mut reader),
                Ok((String::from("abc"), 1..4, String::from("d//e/")))
            );
            assert_eq!(reader.position(), 10);
        }
    }

    mod parse {
        use super::*;

        #[test]
        fn string() {
            let mut reader = Reader::from("/abc/def");
            assert_eq!(
                Substitution::parse_string(&mut reader),
                Ok(Substitution {
                    target: String::from("abc"),
                    replacement: String::from("def"),
                })
            );
            assert_eq!(reader.position(), 8);
        }

        #[test]
        fn valid_regex() {
            let mut reader = Reader::from("/\\d+/def");
            assert_eq!(
                Substitution::parse_regex(&mut reader),
                Ok(Substitution {
                    target: RegexHolder(Regex::new("\\d+").unwrap()),
                    replacement: String::from("def"),
                })
            );
            assert_eq!(reader.position(), 8);
        }

        #[test]
        fn invalid_regex() {
            let mut reader = Reader::from("/[0-9+/def");
            assert_eq!(
                Substitution::parse_regex(&mut reader),
                Err(Error {
                    kind: ErrorKind::SubstitutionRegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 1..6,
                })
            );
            assert_eq!(reader.position(), 10);
        }
    }

    mod display {
        use super::*;

        #[test]
        fn string() {
            assert_eq!(
                Substitution {
                    target: String::from("abc"),
                    replacement: String::from("def")
                }
                .to_string(),
                "'abc' with 'def'"
            );
        }

        #[test]
        fn regex() {
            assert_eq!(
                Substitution {
                    target: Regex::new("([a-z]+)").unwrap(),
                    replacement: String::from("_$1_")
                }
                .to_string(),
                "'([a-z]+)' with '_$1_'"
            );
        }
    }
}
