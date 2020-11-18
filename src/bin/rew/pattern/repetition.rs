use crate::pattern::char::Char;
use crate::pattern::number::parse_number;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Repetition {
    pub count: usize,
    pub value: String,
}

impl Repetition {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        if reader.peek().is_none() {
            return Err(Error {
                kind: ErrorKind::ExpectedRepetition,
                range: reader.position()..reader.position(),
            });
        }

        let count = parse_number(reader)?;
        let position = reader.position();

        if let Some(delimiter) = reader.read_char() {
            if delimiter.is_ascii_digit() {
                Err(Error {
                    kind: ErrorKind::RepetitionDigitDelimiter(delimiter),
                    range: position..reader.position(),
                })
            } else {
                let value = Char::join(reader.read_to_end());
                Ok(Self { count, value })
            }
        } else {
            Err(Error {
                kind: ErrorKind::RepetitionWithoutDelimiter,
                range: reader.position()..reader.end(),
            })
        }
    }
}

impl fmt::Display for Repetition {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}x '{}'", self.count, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::parse::{Error, ErrorKind};
    use crate::pattern::reader::Reader;

    #[test]
    fn parse_empty_error() {
        let mut reader = Reader::from("");
        assert_eq!(
            Repetition::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::ExpectedRepetition,
                range: 0..0
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_invalid_count_error() {
        let mut reader = Reader::from("ab");
        assert_eq!(
            Repetition::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::ExpectedNumber,
                range: 0..2
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_missing_delimiter_error() {
        let mut reader = Reader::from("12");
        assert_eq!(
            Repetition::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::RepetitionWithoutDelimiter,
                range: 2..2
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn parse_digit_delimiter_error() {
        let mut reader = Reader::from("010");
        assert_eq!(
            Repetition::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::RepetitionDigitDelimiter('1'),
                range: 1..2
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn parse_no_value() {
        let mut reader = Reader::from("12:");
        assert_eq!(
            Repetition::parse(&mut reader),
            Ok(Repetition {
                count: 12,
                value: String::new()
            })
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn parse_some_value() {
        let mut reader = Reader::from("12:ab");
        assert_eq!(
            Repetition::parse(&mut reader),
            Ok(Repetition {
                count: 12,
                value: String::from("ab")
            })
        );
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn display() {
        assert_eq!(
            Repetition {
                count: 5,
                value: String::from("abc")
            }
            .to_string(),
            "5x 'abc'"
        );
    }
}
