use crate::pattern::char::{AsChar, Char};
use crate::pattern::escape::escape_str;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::repetition::Repetition;
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Padding {
    Fixed(String),
    Repeated(Repetition),
}

impl Padding {
    pub fn parse(reader: &mut Reader<Char>, fixed_prefix: char) -> Result<Self> {
        let position = reader.position();
        match reader.peek() {
            Some(prefix) => match prefix.as_char() {
                '0'..='9' => Ok(Self::Repeated(Repetition::parse(reader)?)),
                prefix if prefix == fixed_prefix => {
                    reader.seek();
                    Ok(Self::Fixed(reader.read_to_end().to_string()))
                }
                _ => Err(Error {
                    kind: ErrorKind::PaddingPrefixInvalid(fixed_prefix, Some(prefix.clone())),
                    range: position..(position + prefix.len_utf8()),
                }),
            },
            None => Err(Error {
                kind: ErrorKind::PaddingPrefixInvalid(fixed_prefix, None),
                range: position..position,
            }),
        }
    }

    pub fn apply_left(&self, mut value: String) -> String {
        for char in self.expand().chars().rev().skip(value.len()) {
            value.insert(0, char);
        }
        value
    }

    pub fn apply_right(&self, mut value: String) -> String {
        for char in self.expand().chars().skip(value.len()) {
            value.push(char);
        }
        value
    }

    fn expand(&self) -> Cow<String> {
        match self {
            Self::Fixed(value) => Cow::Borrowed(value),
            Self::Repeated(repetition) => Cow::Owned(repetition.expand()),
        }
    }
}

impl fmt::Display for Padding {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Fixed(value) => write!(formatter, "'{}'", escape_str(&value)),
            Self::Repeated(repetition) => write!(formatter, "{}", repetition),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;
    use test_case::test_case;

    #[test_case("", ErrorKind::PaddingPrefixInvalid('<', None), 0..0; "no prefix")]
    #[test_case(">abc", ErrorKind::PaddingPrefixInvalid('<', Some(Char::Raw('>'))), 0..1; "invalid prefix")]
    fn parse_err(input: &str, kind: ErrorKind, range: Range<usize>) {
        assert_eq!(
            Padding::parse(&mut Reader::from(input), '<'),
            Err(Error { kind, range })
        );
    }

    mod fixed {
        use super::*;
        use test_case::test_case;

        #[test_case("<", ""; "empty")]
        #[test_case("<abc", "abc"; "nonempty")]
        fn parse(input: &str, padding: &str) {
            assert_eq!(
                Padding::parse(&mut Reader::from(input), '<'),
                Ok(Padding::Fixed(String::from(padding)))
            );
        }

        #[test_case("", "", ""; "empty with empty")]
        #[test_case("", "0123", "0123"; "empty with nonempty")]
        #[test_case("abcd", "", "abcd"; "nonempty with empty")]
        #[test_case("abcd", "0123", "abcd"; "nonempty same length")]
        #[test_case("ab", "0123", "01ab"; "shorter with longer")]
        fn apply_left(input: &str, padding: &str, output: &str) {
            assert_eq!(
                Padding::Fixed(String::from(padding)).apply_left(String::from(input)),
                String::from(output)
            );
        }

        #[test_case("", "", ""; "empty with empty")]
        #[test_case("", "0123", "0123"; "empty with nonempty")]
        #[test_case("abcd", "", "abcd"; "nonempty with empty")]
        #[test_case("abcd", "0123", "abcd"; "nonempty same length")]
        #[test_case("ab", "0123", "ab23"; "shorter with longer")]
        fn apply_right(input: &str, padding: &str, output: &str) {
            assert_eq!(
                Padding::Fixed(String::from(padding)).apply_right(String::from(input)),
                String::from(output)
            );
        }

        #[test]
        fn display() {
            assert_eq!(Padding::Fixed(String::from("abc")).to_string(), "'abc'");
        }
    }

    mod repeated {
        use super::*;
        use test_case::test_case;

        #[test_case("10:", 10, ""; "empty")]
        #[test_case("10:abc", 10, "abc"; "nonempty")]
        fn parse(input: &str, count: usize, padding: &str) {
            assert_eq!(
                Padding::parse(&mut Reader::from(input), '<'),
                Ok(Padding::Repeated(Repetition {
                    count,
                    value: String::from(padding)
                }))
            );
        }

        #[test_case("", 2, "", ""; "empty with empty")]
        #[test_case("", 2, "012", "012012"; "empty with nonempty")]
        #[test_case("abc", 2, "", "abc"; "nonempty with empty")]
        #[test_case("abc", 1, "012", "abc"; "nonempty same length")]
        #[test_case("ab", 2, "012", "0120ab"; "shorter with longer")]
        fn apply_left(input: &str, count: usize, padding: &str, output: &str) {
            assert_eq!(
                Padding::Repeated(Repetition {
                    count,
                    value: String::from(padding)
                })
                .apply_left(String::from(input)),
                String::from(output)
            );
        }

        #[test_case("", 2, "", ""; "empty with empty")]
        #[test_case("", 2, "012", "012012"; "empty with nonempty")]
        #[test_case("abc", 2, "", "abc"; "nonempty with empty")]
        #[test_case("abc", 1, "012", "abc"; "nonempty same length")]
        #[test_case("ab", 2, "012", "ab2012"; "shorter with longer")]
        fn apply_right(input: &str, count: usize, padding: &str, output: &str) {
            assert_eq!(
                Padding::Repeated(Repetition {
                    count,
                    value: String::from(padding)
                })
                .apply_right(String::from(input)),
                String::from(output)
            );
        }

        #[test]
        fn display() {
            assert_eq!(
                Padding::Repeated(Repetition {
                    count: 5,
                    value: String::from("abc")
                })
                .to_string(),
                "5x 'abc'"
            );
        }
    }
}
