use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use crate::utils::{AnyString, Empty};
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::fmt;
use std::ops::Range;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$(\d+)").unwrap();
}

#[derive(Debug, PartialEq)]
pub struct Substitution<T> {
    pub target: T,
    pub replacement: String,
}

impl Substitution<Empty> {
    pub fn parse_empty(reader: &mut Reader<Char>) -> Result<Self> {
        Ok(Self {
            target: Empty,
            replacement: reader.read_to_end().to_string(),
        })
    }

    pub fn replace(&self, mut value: String) -> String {
        if value.is_empty() {
            value.push_str(&self.replacement);
        }
        value
    }
}

impl Substitution<String> {
    pub fn parse_string(reader: &mut Reader<Char>) -> Result<Self> {
        let (target, _, replacement) = parse_target_and_replacement(reader)?;
        Ok(Self {
            target,
            replacement,
        })
    }

    pub fn replace_first(&self, value: &str) -> String {
        value.replacen(&self.target, &self.replacement, 1)
    }

    pub fn replace_all(&self, value: &str) -> String {
        value.replace(&self.target, &self.replacement)
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

    pub fn replace_first(&self, value: &str) -> String {
        let replacement = add_capture_group_brackets(&self.replacement);
        self.target
            .0
            .replace(value, replacement.as_ref())
            .to_string()
    }

    pub fn replace_all(&self, value: &str) -> String {
        let replacement = add_capture_group_brackets(&self.replacement);
        self.target
            .0
            .replace_all(value, replacement.as_ref())
            .to_string()
    }
}

pub fn parse_target_and_replacement(
    reader: &mut Reader<Char>,
) -> Result<(String, Range<usize>, String)> {
    if let Some(delimiter) = reader.read().cloned() {
        let target_start = reader.position();
        let target = reader.read_until(&delimiter);
        let target_end = target_start + target.len_utf8();

        if target.is_empty() {
            return Err(Error {
                kind: ErrorKind::SubstitutionWithoutTarget(delimiter),
                range: target_start..target_end,
            });
        }

        Ok((
            target.to_string(),
            target_start..target_end,
            reader.read_to_end().to_string(),
        ))
    } else {
        Err(Error {
            kind: ErrorKind::ExpectedSubstitution,
            range: reader.position()..reader.end(),
        })
    }
}

fn add_capture_group_brackets(string: &str) -> Cow<str> {
    if string.contains('$') {
        CAPTURE_GROUP_VAR_REGEX.replace_all(string, r"$${${1}}")
    } else {
        Cow::Borrowed(string)
    }
}

impl fmt::Display for Substitution<Empty> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "empty with '{}'", self.replacement)
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

    mod empty {
        use super::*;

        mod replace {
            use super::*;

            #[test]
            fn empty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: Empty,
                        replacement: String::new()
                    }
                    .replace(String::new()),
                    String::new()
                );
            }

            #[test]
            fn empty_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: Empty,
                        replacement: String::from("def")
                    }
                    .replace(String::new()),
                    String::from("def")
                );
            }

            #[test]
            fn nonempty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: Empty,
                        replacement: String::new()
                    }
                    .replace(String::from("abc")),
                    String::from("abc")
                );
            }

            #[test]
            fn nonempty_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: Empty,
                        replacement: String::from("def")
                    }
                    .replace(String::from("abc")),
                    String::from("abc")
                );
            }
        }

        #[test]
        fn display() {
            assert_eq!(
                Substitution {
                    target: Empty,
                    replacement: String::from("abc")
                }
                .to_string(),
                "empty with 'abc'"
            );
        }
    }

    mod string {
        use super::*;

        #[test]
        fn parse() {
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

        mod replace_first {
            use super::*;

            #[test]
            fn empty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::new(),
                    }
                    .replace_first(""),
                    String::new()
                );
            }

            #[test]
            fn empty_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::from("x"),
                    }
                    .replace_first(""),
                    String::new()
                );
            }

            #[test]
            fn none_with_empty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::new(),
                    }
                    .replace_first("cd"),
                    String::from("cd")
                );
            }

            #[test]
            fn none_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::from("x"),
                    }
                    .replace_first("cd"),
                    String::from("cd")
                );
            }

            #[test]
            fn first_with_empty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::new(),
                    }
                    .replace_first("abcd_abcd"),
                    String::from("cd_abcd")
                );
            }

            #[test]
            fn first_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::from("x"),
                    }
                    .replace_first("abcd_abcd"),
                    String::from("xcd_abcd")
                );
            }
        }

        mod replace_all {
            use super::*;

            #[test]
            fn empty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::new(),
                    }
                    .replace_all(""),
                    String::new()
                );
            }

            #[test]
            fn empty_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::from("x"),
                    }
                    .replace_all(""),
                    String::new()
                );
            }

            #[test]
            fn none_with_empty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::new(),
                    }
                    .replace_all("cd"),
                    String::from("cd")
                );
            }

            #[test]
            fn none_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::from("x"),
                    }
                    .replace_all("cd"),
                    String::from("cd")
                );
            }

            #[test]
            fn all_with_empty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::new(),
                    }
                    .replace_all("abcd_abcd"),
                    String::from("cd_cd")
                );
            }

            #[test]
            fn all_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: String::from("ab"),
                        replacement: String::from("x"),
                    }
                    .replace_all("abcd_abcd"),
                    String::from("xcd_xcd")
                );
            }
        }

        #[test]
        fn display() {
            assert_eq!(
                Substitution {
                    target: String::from("abc"),
                    replacement: String::from("def")
                }
                .to_string(),
                "'abc' with 'def'"
            );
        }
    }

    mod regex {
        use super::*;

        mod parse {
            use super::*;

            #[test]
            fn valid() {
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
            fn invalid() {
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

        mod replace_first {
            use super::*;

            #[test]
            fn empty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("\\d+").unwrap()),
                        replacement: String::new()
                    }
                    .replace_first(""),
                    String::new()
                );
            }

            #[test]
            fn empty_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("(\\d)(\\d+)").unwrap()),
                        replacement: String::from("_$2$1_")
                    }
                    .replace_first(""),
                    String::new()
                );
            }

            #[test]
            fn none_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("\\d+").unwrap()),
                        replacement: String::new()
                    }
                    .replace_first(""),
                    String::new()
                );
            }

            #[test]
            fn none_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("(\\d)(\\d+)").unwrap()),
                        replacement: String::from("_$2$1_")
                    }
                    .replace_first("abc"),
                    String::from("abc")
                );
            }

            #[test]
            fn first_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("\\d+").unwrap()),
                        replacement: String::new()
                    }
                    .replace_first("abc123def456"),
                    String::from("abcdef456")
                );
            }

            #[test]
            fn first_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("(\\d)(\\d+)").unwrap()),
                        replacement: String::from("_$2$1_")
                    }
                    .replace_first("abc123def456"),
                    String::from("abc_231_def456")
                );
            }
        }

        mod replace_all {
            use super::*;

            #[test]
            fn empty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("\\d+").unwrap()),
                        replacement: String::new()
                    }
                    .replace_all(""),
                    String::new()
                );
            }

            #[test]
            fn empty_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("(\\d)(\\d+)").unwrap()),
                        replacement: String::from("_$2$1_")
                    }
                    .replace_all(""),
                    String::new()
                );
            }

            #[test]
            fn none_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("\\d+").unwrap()),
                        replacement: String::new()
                    }
                    .replace_all("abc"),
                    String::from("abc")
                );
            }

            #[test]
            fn none_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("(\\d)(\\d+)").unwrap()),
                        replacement: String::from("_$2$1_")
                    }
                    .replace_all("abc"),
                    String::from("abc")
                );
            }

            #[test]
            fn all_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("\\d+").unwrap()),
                        replacement: String::new()
                    }
                    .replace_all("abc123def456"),
                    String::from("abcdef")
                );
            }

            #[test]
            fn all_with_nonempty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder(Regex::new("(\\d)(\\d+)").unwrap()),
                        replacement: String::from("_$2$1_")
                    }
                    .replace_all("abc123def456"),
                    String::from("abc_231_def_564_")
                );
            }
        }

        #[test]
        fn display() {
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
}
