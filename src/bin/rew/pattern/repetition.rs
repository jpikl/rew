use crate::pattern::char::Char;
use crate::pattern::integer::parse_integer;
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

        let count = parse_integer(reader)?;
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

    pub fn expand(&self) -> String {
        self.value.repeat(self.count)
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

    mod parse {
        use super::*;

        #[test]
        fn empty() {
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
        fn invalid_count() {
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
        fn missing_delimiter() {
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
        fn digit_delimiter() {
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
        fn empty_value() {
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
        fn nonempty_value() {
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
    }

    mod expand {
        use super::*;

        #[test]
        fn empty_zero_times() {
            assert_eq!(
                Repetition {
                    count: 0,
                    value: String::new()
                }
                .expand(),
                String::new()
            );
        }

        #[test]
        fn empty_multiple_times() {
            assert_eq!(
                Repetition {
                    count: 2,
                    value: String::new()
                }
                .expand(),
                String::new()
            );
        }

        #[test]
        fn nonempty_zero_times() {
            assert_eq!(
                Repetition {
                    count: 0,
                    value: String::from("ab")
                }
                .expand(),
                String::new()
            );
        }

        #[test]
        fn nonempty_multiple_times() {
            assert_eq!(
                Repetition {
                    count: 2,
                    value: String::from("ab")
                }
                .expand(),
                String::from("abab")
            );
        }
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
