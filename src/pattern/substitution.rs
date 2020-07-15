use crate::pattern::char::{AsChar, Char};
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::reader::Reader;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Substitution {
    pub value: String,
    pub replacement: String,
}

impl Substitution {
    pub fn parse(reader: &mut Reader<Char>) -> ParseResult<Self> {
        if let Some(separator) = reader.read().cloned() {
            let mut value = String::new();
            let position = reader.position();

            while let Some(ch) = reader.read_char() {
                if ch == separator.as_char() {
                    break;
                } else {
                    value.push(ch);
                }
            }

            if value.is_empty() {
                return Err(ParseError {
                    kind: ParseErrorKind::SubstituteWithoutValue(separator),
                    range: position..position,
                });
            }

            Ok(Self {
                value,
                replacement: Char::join(reader.read_to_end()),
            })
        } else {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                range: reader.position()..reader.end(),
            })
        }
    }
}

impl fmt::Display for Substitution {
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
            Substitution::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                range: 0..0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn no_value_error() {
        let mut reader = Reader::from("/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::SubstituteWithoutValue(Char::Raw('/')),
                range: 1..1,
            })
        );
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn empty_value_error() {
        let mut reader = Reader::from("//");
        assert_eq!(
            Substitution::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::SubstituteWithoutValue(Char::Raw('/')),
                range: 1..1,
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn value_no_replacement() {
        let mut reader = Reader::from("/a");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("a"),
                replacement: String::from("")
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn long_value_no_replacement() {
        let mut reader = Reader::from("/abc");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("abc"),
                replacement: String::from("")
            })
        );
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn value_empty_replacement() {
        let mut reader = Reader::from("/a/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("a"),
                replacement: String::from("")
            })
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn long_value_empty_replacement() {
        let mut reader = Reader::from("/abc/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("abc"),
                replacement: String::from("")
            })
        );
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn value_replacement() {
        let mut reader = Reader::from("/a/d");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("a"),
                replacement: String::from("d")
            })
        );
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn long_value_replacement() {
        let mut reader = Reader::from("/abc/def");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("abc"),
                replacement: String::from("def")
            })
        );
        assert_eq!(reader.position(), 8);
    }

    #[test]
    fn value_replacement_with_redundant_separators() {
        let mut reader = Reader::from("/abc/d//e/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: String::from("abc"),
                replacement: String::from("d//e/")
            })
        );
        assert_eq!(reader.position(), 10);
    }
}
