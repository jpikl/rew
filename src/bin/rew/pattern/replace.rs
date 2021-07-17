use crate::pattern::char::Char;
use crate::pattern::escape::escape_str;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::regex::{add_capture_group_brackets, RegexHolder};
use crate::pattern::utils::Empty;
use std::convert::{TryFrom, TryInto};
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

            match target.to_string().try_into() {
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
    use crate::pattern::error::ErrorRange;

    mod empty {
        use super::*;
        use test_case::test_case;

        #[test_case("",    ""    ; "empty")]
        #[test_case("abc", "abc" ; "nonempty")]
        fn parse(input: &str, replacement: &str) {
            assert_eq!(
                EmptySubstitution::parse(&mut Reader::from(input)),
                Ok(EmptySubstitution {
                    target: Empty,
                    replacement: replacement.into()
                })
            );
        }

        #[test_case("",    "",    ""    ; "empty with empty")]
        #[test_case("",    "def", "def" ; "empty with nonempty")]
        #[test_case("abc", "",    "abc" ; "nonempty with empty")]
        #[test_case("abc", "def", "abc" ; "nonempty with nonempty")]
        fn replace(input: &str, replacement: &str, output: &str) {
            assert_eq!(
                Substitution {
                    target: Empty,
                    replacement: replacement.into()
                }
                .replace(input.into()),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                Substitution {
                    target: Empty,
                    replacement: "abc".into()
                }
                .to_string(),
                "empty with 'abc'"
            );
        }
    }

    mod string {
        use super::*;
        use test_case::test_case;

        mod parse {
            use super::*;
            use test_case::test_case;

            #[test_case("",   0..0, ErrorKind::ExpectedSubstitution                  ; "empty")]
            #[test_case("/",  1..1, ErrorKind::SubstitutionWithoutTarget('/'.into()) ; "no target")]
            #[test_case("//", 1..1, ErrorKind::SubstitutionWithoutTarget('/'.into()) ; "empty target")]
            fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
                assert_eq!(
                    StringSubstitution::parse(&mut Reader::from(input)),
                    Err(Error { kind, range })
                );
            }

            #[test_case("/a",         "a",   ""      ; "short target no replacement")]
            #[test_case("/abc",       "abc", ""      ; "long target no replacement")]
            #[test_case("/a/",        "a",   ""      ; "short target empty replacement")]
            #[test_case("/abc/",      "abc", ""      ; "long target empty replacement")]
            #[test_case("/a/d",       "a",   "d"     ; "short target short replacement")]
            #[test_case("/abc/def",   "abc", "def"   ; "long target long replacement")]
            #[test_case("/abc/d//e/", "abc", "d//e/" ; "long target long replacement containing delimiter")]
            fn ok(input: &str, target: &str, replacement: &str) {
                assert_eq!(
                    StringSubstitution::parse(&mut Reader::from(input)),
                    Ok(Substitution {
                        target: target.into(),
                        replacement: replacement.into(),
                    })
                );
            }
        }

        #[test_case("",          "ab", "",  ""         ; "empty with empty")]
        #[test_case("",          "ab", "x", ""         ; "empty with nonempty")]
        #[test_case("cd",        "ab", "",  "cd"       ; "none with empty")]
        #[test_case("cd",        "ab", "x", "cd"       ; "none with nonempty")]
        #[test_case("abcd_abcd", "ab", "",  "cd_abcd"  ; "first with empty")]
        #[test_case("abcd_abcd", "ab", "x", "xcd_abcd" ; "first with nonempty")]
        fn replace_first(input: &str, target: &str, replacement: &str, output: &str) {
            assert_eq!(
                StringSubstitution {
                    target: target.into(),
                    replacement: replacement.into(),
                }
                .replace_first(input),
                output
            );
        }

        #[test_case("",          "ab", "",  ""        ; "empty with empty")]
        #[test_case("",          "ab", "x", ""        ; "empty with nonempty")]
        #[test_case("cd",        "ab", "",  "cd"      ; "none with empty")]
        #[test_case("cd",        "ab", "x", "cd"      ; "none with nonempty")]
        #[test_case("abcd_abcd", "ab", "",  "cd_cd"   ; "all with empty")]
        #[test_case("abcd_abcd", "ab", "x", "xcd_xcd" ; "all with nonempty")]
        fn replace_all(input: &str, target: &str, replacement: &str, output: &str) {
            assert_eq!(
                StringSubstitution {
                    target: target.into(),
                    replacement: replacement.into(),
                }
                .replace_all(input),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                StringSubstitution {
                    target: "abc".into(),
                    replacement: "def".into()
                }
                .to_string(),
                "'abc' with 'def'"
            );
        }
    }

    mod regex {
        extern crate regex;
        use super::*;
        use test_case::test_case;

        mod parse {
            use super::*;
            use crate::pattern::utils::AnyString;
            use test_case::test_case;

            #[test_case("",           0..0, ErrorKind::ExpectedSubstitution                  ; "empty")]
            #[test_case("/",          1..1, ErrorKind::SubstitutionWithoutTarget('/'.into()) ; "no target")]
            #[test_case("//",         1..1, ErrorKind::SubstitutionWithoutTarget('/'.into()) ; "empty target")]
            #[test_case("/[0-9+/def", 1..6, ErrorKind::RegexInvalid(AnyString::any())        ; "invalid regex")]
            fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
                assert_eq!(
                    RegexSubstitution::parse(&mut Reader::from(input)),
                    Err(Error { kind, range })
                );
            }

            #[test_case("/(\\d+)",      "(\\d+)", ""     ; "no replacement")]
            #[test_case("/(\\d+)/",     "(\\d+)", ""     ; "empty replacement")]
            #[test_case("/(\\d+)/_$1_", "(\\d+)", "_$1_" ; "nonempty replacement")]
            #[test_case("/(\\d+)//$1/", "(\\d+)", "/$1/" ; "replacement containing delimiter")]
            fn ok(input: &str, target: &str, replacement: &str) {
                assert_eq!(
                    RegexSubstitution::parse(&mut Reader::from(input)),
                    Ok(Substitution {
                        target: target.into(),
                        replacement: replacement.into(),
                    })
                );
            }
        }

        #[test_case("",             "\\d+",        "",       ""               ; "empty with empty")]
        #[test_case("",             "(\\d)(\\d+)", "_$2$1_", ""               ; "empty with nonempty")]
        #[test_case("abc",          "\\d+",        "",       "abc"            ; "none with empty")]
        #[test_case("abc",          "(\\d)(\\d+)", "_$2$1_", "abc"            ; "none with nonempty")]
        #[test_case("abc123def456", "\\d+",        "",       "abcdef456"      ; "first with empty")]
        #[test_case("abc123def456", "(\\d)(\\d+)", "_$2$1_", "abc_231_def456" ; "first with nonempty")]
        fn replace_first(input: &str, target: &str, replacement: &str, output: &str) {
            assert_eq!(
                RegexSubstitution {
                    target: target.into(),
                    replacement: replacement.into(),
                }
                .replace_first(input),
                output
            );
        }

        #[test_case("",             "\\d+",        "",       ""                 ; "empty with empty")]
        #[test_case("",             "(\\d)(\\d+)", "_$2$1_", ""                 ; "empty with nonempty")]
        #[test_case("abc",          "\\d+",        "",       "abc"              ; "none with empty")]
        #[test_case("abc",          "(\\d)(\\d+)", "_$2$1_", "abc"              ; "none with nonempty")]
        #[test_case("abc123def456", "\\d+",        "",       "abcdef"           ; "all with empty")]
        #[test_case("abc123def456", "(\\d)(\\d+)", "_$2$1_", "abc_231_def_564_" ; "all with nonempty")]
        fn replace_all(input: &str, target: &str, replacement: &str, output: &str) {
            assert_eq!(
                RegexSubstitution {
                    target: target.into(),
                    replacement: replacement.into(),
                }
                .replace_all(input),
                output
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                RegexSubstitution {
                    target: "([a-z]+)".into(),
                    replacement: "_$1_".into()
                }
                .to_string(),
                "'([a-z]+)' with '_$1_'"
            );
        }
    }
}
