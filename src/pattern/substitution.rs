use crate::pattern::char::Char;
use crate::pattern::error::{ParseError, ParseErrorKind};
use crate::pattern::parse::ParseResult;
use crate::pattern::reader::Reader;

#[derive(Debug, PartialEq)]
pub struct Substitution {
    pub value: String,
    pub replacement: String,
}

impl Substitution {
    pub fn parse(reader: &mut Reader) -> ParseResult<Self> {
        if let Some(separator) = reader.read().cloned() {
            let mut value = String::new();
            let value_position = reader.position();

            while let Some(ch) = reader.read_value() {
                if ch == separator.value() {
                    break;
                } else {
                    value.push(ch);
                }
            }

            if value.is_empty() {
                return Err(ParseError {
                    kind: ParseErrorKind::SubstituteWithoutValue(separator),
                    start: value_position,
                    end: value_position,
                });
            }

            Ok(Self {
                value,
                replacement: Char::join(reader.read_to_end()),
            })
        } else {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                start: reader.position(),
                end: reader.end(),
            })
        }
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
                start: 0,
                end: 0,
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
                start: 1,
                end: 1,
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
                start: 1,
                end: 1,
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
                value: "a".to_string(),
                replacement: "".to_string()
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
                value: "abc".to_string(),
                replacement: "".to_string()
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
                value: "a".to_string(),
                replacement: "".to_string()
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
                value: "abc".to_string(),
                replacement: "".to_string()
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
                value: "a".to_string(),
                replacement: "d".to_string()
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
                value: "abc".to_string(),
                replacement: "def".to_string()
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
                value: "abc".to_string(),
                replacement: "d//e/".to_string()
            })
        );
        assert_eq!(reader.position(), 10);
    }
}
