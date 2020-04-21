use crate::pattern::char::Char;
use crate::pattern::error::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::number::parse_usize;
use crate::pattern::reader::Reader;

#[derive(Debug, PartialEq)]
pub struct Range {
    pub offset: usize,
    pub length: usize, // Zero length means unlimited
}

const DIVIDER: char = '-';

impl Range {
    pub fn parse(reader: &mut Reader) -> ParseResult<Self> {
        let range = match reader.peek_value() {
            Some('0'..='9') => {
                let position = reader.position();
                let offset = parse_offset(reader)?;
                if let Some(DIVIDER) = reader.peek_value() {
                    reader.read_value();
                    if let Some('0'..='9') = reader.peek_value() {
                        let length = parse_length(reader, offset, position)?;
                        Ok(Self { offset, length })
                    } else {
                        Ok(Self { offset, length: 0 })
                    }
                } else {
                    Ok(Self { offset, length: 1 })
                }
            }
            Some(DIVIDER) => {
                reader.read_value();
                if let Some('0'..='9') = reader.peek_value() {
                    let length = parse_length(reader, 0, 0)?;
                    Ok(Self { offset: 0, length })
                } else {
                    Ok(Self {
                        offset: 0,
                        length: 0,
                    })
                }
            }
            Some(_) => Err(ParseError {
                kind: ParseErrorKind::RangeInvalid(Char::join(reader.peek_to_end())),
                start: reader.position(),
                end: reader.end(),
            }),
            None => Err(ParseError {
                kind: ParseErrorKind::ExpectedRange,
                start: reader.position(),
                end: reader.end(),
            }),
        }?;

        if reader.is_end() {
            Ok(range)
        } else {
            Err(ParseError {
                kind: ParseErrorKind::RangeUnexpectedChars(Char::join(reader.peek_to_end())),
                start: reader.position(),
                end: reader.end(),
            })
        }
    }
}

fn parse_offset(reader: &mut Reader) -> ParseResult<usize> {
    let position = reader.position();
    let index = parse_usize(reader)?;

    if index < 1 {
        Err(ParseError {
            kind: ParseErrorKind::RangeIndexZero,
            start: position,
            end: reader.position(),
        })
    } else {
        Ok(index - 1)
    }
}

fn parse_length(reader: &mut Reader, offset: usize, offset_position: usize) -> ParseResult<usize> {
    let position = reader.position();
    let index = parse_usize(reader)?;

    if index < 1 {
        Err(ParseError {
            kind: ParseErrorKind::RangeIndexZero,
            start: position,
            end: reader.position(),
        })
    } else if index <= offset {
        Err(ParseError {
            kind: ParseErrorKind::RangeEndBeforeStart(index, offset + 1),
            start: offset_position,
            end: reader.position(),
        })
    } else {
        Ok(index - offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_error() {
        let mut reader = Reader::from("");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedRange,
                start: 0,
                end: 0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn invalid_error() {
        let mut reader = Reader::from("ab");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeInvalid("ab".to_string()),
                start: 0,
                end: 2,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn full() {
        let mut reader = Reader::from("-");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 0
            })
        );
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn zero_start_error() {
        let mut reader = Reader::from("0-");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeIndexZero,
                start: 0,
                end: 1,
            })
        );
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn start() {
        let mut reader = Reader::from("1-");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 0
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn zero_end_error() {
        let mut reader = Reader::from("-0");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeIndexZero,
                start: 1,
                end: 2,
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn end() {
        let mut reader = Reader::from("-1");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 1
            })
        );
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn start_greater_than_end_error() {
        let mut reader = Reader::from("10-5");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeEndBeforeStart(5, 10),
                start: 0,
                end: 4,
            })
        );
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn start_equals_to_end() {
        let mut reader = Reader::from("5-5");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 4,
                length: 1
            })
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn start_less_than_end() {
        let mut reader = Reader::from("4-5");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 3,
                length: 2
            })
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn united_start_and_end() {
        let mut reader = Reader::from("100");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 99,
                length: 1
            })
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn unexpected_chars_error() {
        let mut reader = Reader::from("1ab");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeUnexpectedChars("ab".to_string()),
                start: 1,
                end: 3,
            })
        );
        assert_eq!(reader.position(), 1);
    }
}
