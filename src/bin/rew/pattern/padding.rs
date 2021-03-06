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

    mod parse {
        use super::*;

        #[test]
        fn empty() {
            let mut reader = Reader::from("");
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Err(Error {
                    kind: ErrorKind::PaddingPrefixInvalid('<', None),
                    range: 0..0,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn invalid_prefix() {
            let mut reader = Reader::from(">abc");
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Err(Error {
                    kind: ErrorKind::PaddingPrefixInvalid('<', Some(Char::Raw('>'))),
                    range: 0..1,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn invalid_escaped_prefix() {
            let mut reader = Reader::new(vec![Char::Escaped('x', ['%', 'x'])]);
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Err(Error {
                    kind: ErrorKind::PaddingPrefixInvalid(
                        '<',
                        Some(Char::Escaped('x', ['%', 'x']))
                    ),
                    range: 0..2,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn fixed_empty() {
            let mut reader = Reader::from("<");
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Ok(Padding::Fixed(String::new()))
            );
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn fixed_nonempty() {
            let mut reader = Reader::from("<abc");
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Ok(Padding::Fixed(String::from("abc")))
            );
            assert_eq!(reader.position(), 4);
        }

        #[test]
        fn repeated_empty() {
            let mut reader = Reader::from("10:");
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Ok(Padding::Repeated(Repetition {
                    count: 10,
                    value: String::new()
                }))
            );
            assert_eq!(reader.position(), 3);
        }

        #[test]
        fn repeated_nonempty() {
            let mut reader = Reader::from("10:abc");
            assert_eq!(
                Padding::parse(&mut reader, '<'),
                Ok(Padding::Repeated(Repetition {
                    count: 10,
                    value: String::from("abc")
                }))
            );
            assert_eq!(reader.position(), 6);
        }
    }

    mod apply_left {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(
                Padding::Fixed(String::new()).apply_left(String::new()),
                String::new()
            );
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(
                Padding::Fixed(String::from("0123")).apply_left(String::new()),
                String::from("0123")
            );
        }

        #[test]
        fn nonempty_with_empty() {
            assert_eq!(
                Padding::Fixed(String::new()).apply_left(String::from("abcd")),
                String::from("abcd")
            );
        }

        #[test]
        fn shorter_with_longer() {
            assert_eq!(
                Padding::Fixed(String::from("0123")).apply_left(String::from("ab")),
                String::from("01ab")
            );
        }

        #[test]
        fn longer_with_shorter() {
            assert_eq!(
                Padding::Fixed(String::from("0123")).apply_left(String::from("abcd")),
                String::from("abcd")
            );
        }

        #[test]
        fn repeated() {
            assert_eq!(
                Padding::Repeated(Repetition {
                    count: 3,
                    value: String::from("01")
                })
                .apply_left(String::from("abc")),
                String::from("010abc")
            );
        }
    }

    mod apply_right {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(
                Padding::Fixed(String::new()).apply_right(String::new()),
                String::new()
            );
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(
                Padding::Fixed(String::from("0123")).apply_right(String::new()),
                String::from("0123")
            );
        }

        #[test]
        fn nonempty_with_empty() {
            assert_eq!(
                Padding::Fixed(String::new()).apply_right(String::from("abcd")),
                String::from("abcd")
            );
        }

        #[test]
        fn shorter_with_longer() {
            assert_eq!(
                Padding::Fixed(String::from("0123")).apply_right(String::from("ab")),
                String::from("ab23")
            );
        }

        #[test]
        fn longer_with_shorter() {
            assert_eq!(
                Padding::Fixed(String::from("0123")).apply_right(String::from("abcd")),
                String::from("abcd")
            );
        }

        #[test]
        fn repeated() {
            assert_eq!(
                Padding::Repeated(Repetition {
                    count: 3,
                    value: String::from("01")
                })
                .apply_right(String::from("abc")),
                String::from("abc101")
            );
        }
    }

    mod display {
        use super::*;

        #[test]
        fn fixed() {
            assert_eq!(Padding::Fixed(String::from("abc")).to_string(), "'abc'");
        }

        #[test]
        fn repeated() {
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
