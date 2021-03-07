use crate::pattern::char::Char;
use crate::pattern::escape::escape_str;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::{add_capture_group_brackets, RegexHolder};
use crate::utils::Empty;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Substitution<T> {
    pub target: T,
    pub replacement: String,
}

pub type EmptySubstitution = Substitution<Empty>;
pub type StringSubstitution = Substitution<String>;
pub type RegexSubstitution = Substitution<RegexHolder>;

impl EmptySubstitution {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
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

impl<E: Into<ErrorKind>, T: TryFrom<String, Error = E>> Substitution<T> {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
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

            match T::try_from(target.to_string()) {
                Ok(target) => Ok(Self {
                    target,
                    replacement: reader.read_to_end().to_string(),
                }),
                Err(error) => Err(Error {
                    kind: error.into(),
                    range: target_start..target_end,
                }),
            }
        } else {
            Err(Error {
                kind: ErrorKind::ExpectedSubstitution,
                range: reader.position()..reader.end(),
            })
        }
    }
}

impl StringSubstitution {
    pub fn replace_first(&self, value: &str) -> String {
        value.replacen(&self.target, &self.replacement, 1)
    }

    pub fn replace_all(&self, value: &str) -> String {
        value.replace(&self.target, &self.replacement)
    }
}

impl RegexSubstitution {
    pub fn replace_first(&self, value: &str) -> String {
        let replacement = add_capture_group_brackets(&self.replacement);
        self.target.replace(value, replacement.as_ref()).to_string()
    }

    pub fn replace_all(&self, value: &str) -> String {
        let replacement = add_capture_group_brackets(&self.replacement);
        self.target
            .replace_all(value, replacement.as_ref())
            .to_string()
    }
}

impl fmt::Display for EmptySubstitution {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "empty with '{}'", escape_str(&self.replacement))
    }
}

impl fmt::Display for StringSubstitution {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "'{}' with '{}'",
            escape_str(&self.target),
            escape_str(&self.replacement)
        )
    }
}

impl fmt::Display for RegexSubstitution {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "'{}' with '{}'",
            self.target,
            escape_str(&self.replacement)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod empty {
        use super::*;

        mod parse {
            use super::*;

            #[test]
            fn empty() {
                let mut reader = Reader::from("");
                assert_eq!(
                    EmptySubstitution::parse(&mut reader),
                    Ok(EmptySubstitution {
                        target: Empty,
                        replacement: String::new()
                    })
                );
                assert_eq!(reader.position(), 0);
            }

            #[test]
            fn nonempty() {
                let mut reader = Reader::from("abc");
                assert_eq!(
                    EmptySubstitution::parse(&mut reader),
                    Ok(EmptySubstitution {
                        target: Empty,
                        replacement: String::from("abc")
                    })
                );
                assert_eq!(reader.position(), 3);
            }
        }

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

        mod parse {
            use super::*;

            #[test]
            fn empty() {
                let mut reader = Reader::from("");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
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
                    StringSubstitution::parse(&mut reader),
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
                    StringSubstitution::parse(&mut reader),
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
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("a"),
                        replacement: String::new(),
                    })
                );
                assert_eq!(reader.position(), 2)
            }

            #[test]
            fn long_target_no_replacement() {
                let mut reader = Reader::from("/abc");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("abc"),
                        replacement: String::new(),
                    })
                );
                assert_eq!(reader.position(), 4);
            }

            #[test]
            fn short_target_empty_replacement() {
                let mut reader = Reader::from("/a/");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("a"),
                        replacement: String::new(),
                    })
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn long_target_empty_replacement() {
                let mut reader = Reader::from("/abc/");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("abc"),
                        replacement: String::new(),
                    })
                );
                assert_eq!(reader.position(), 5);
            }

            #[test]
            fn short_target_short_replacement() {
                let mut reader = Reader::from("/a/d");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("a"),
                        replacement: String::from("d"),
                    })
                );
                assert_eq!(reader.position(), 4);
            }

            #[test]
            fn long_target_long_replacement() {
                let mut reader = Reader::from("/abc/def");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("abc"),
                        replacement: String::from("def"),
                    })
                );
                assert_eq!(reader.position(), 8);
            }

            #[test]
            fn long_target_long_replacement_containing_delimiters() {
                let mut reader = Reader::from("/abc/d//e/");
                assert_eq!(
                    StringSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: String::from("abc"),
                        replacement: String::from("d//e/"),
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
        extern crate regex;
        use super::*;
        use crate::utils::AnyString;

        mod parse {
            use super::*;

            #[test]
            fn valid() {
                let mut reader = Reader::from("/\\d+/def");
                assert_eq!(
                    RegexSubstitution::parse(&mut reader),
                    Ok(Substitution {
                        target: RegexHolder::from("\\d+"),
                        replacement: String::from("def"),
                    })
                );
                assert_eq!(reader.position(), 8);
            }

            #[test]
            fn invalid() {
                let mut reader = Reader::from("/[0-9+/def");
                assert_eq!(
                    RegexSubstitution::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RegexInvalid(AnyString::any()),
                        range: 1..6,
                    })
                );
                assert_eq!(reader.position(), 7);
            }
        }

        mod replace_first {
            use super::*;

            #[test]
            fn empty_with_empty() {
                assert_eq!(
                    Substitution {
                        target: RegexHolder::from("\\d+"),
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
                        target: RegexHolder::from("(\\d)(\\d+)"),
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
                        target: RegexHolder::from("\\d+"),
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
                        target: RegexHolder::from("(\\d)(\\d+)"),
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
                        target: RegexHolder::from("\\d+"),
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
                        target: RegexHolder::from("(\\d)(\\d+)"),
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
                        target: RegexHolder::from("\\d+"),
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
                        target: RegexHolder::from("(\\d)(\\d+)"),
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
                        target: RegexHolder::from("\\d+"),
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
                        target: RegexHolder::from("(\\d)(\\d+)"),
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
                        target: RegexHolder::from("\\d+"),
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
                        target: RegexHolder::from("(\\d)(\\d+)"),
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
                    target: RegexHolder::from("([a-z]+)"),
                    replacement: String::from("_$1_")
                }
                .to_string(),
                "'([a-z]+)' with '_$1_'"
            );
        }
    }
}
