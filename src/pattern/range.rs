use crate::pattern::char::Char;
use crate::pattern::number::parse_usize;
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::reader::Reader;

const DIVIDER: char = '-';

#[derive(Debug, PartialEq)]
pub enum Range {
    Full,
    From(usize),
    FromTo(usize, usize),
    To(usize),
}

impl Range {
    pub fn start(&self) -> Option<usize> {
        match self {
            Range::Full => None,
            Range::From(start) => Some(*start),
            Range::FromTo(start, _) => Some(*start),
            Range::To(_) => None,
        }
    }

    pub fn length(&self) -> Option<usize> {
        match self {
            Range::Full => None,
            Range::From(_) => None,
            Range::FromTo(start, end) => {
                if start > end {
                    panic!("Range start ({}) > end ({})", start, end)
                }
                Some(end - start)
            }
            Range::To(end) => Some(*end),
        }
    }

    pub fn parse(reader: &mut Reader) -> ParseResult<Self> {
        match reader.peek_value() {
            Some('0'..='9') => {
                let position = reader.position();
                let start = parse_index(reader)?;

                if let Some(DIVIDER) = reader.peek_value() {
                    reader.seek();

                    if let Some('0'..='9') = reader.peek_value() {
                        let end = parse_index(reader)?;
                        if start > end {
                            Err(ParseError {
                                kind: ParseErrorKind::RangeStartOverEnd(start + 1, end + 1),
                                start: position,
                                end: reader.position(),
                            })
                        } else {
                            Ok(Range::FromTo(start, end + 1)) // Inclusive end -> exclusive end
                        }
                    } else {
                        Ok(Range::From(start))
                    }
                } else {
                    Ok(Range::FromTo(start, start + 1))
                }
            }

            Some(DIVIDER) => {
                reader.seek();

                if let Some('0'..='9') = reader.peek_value() {
                    let end = parse_index(reader)?;
                    Ok(Range::To(end + 1)) // Inclusive end -> exclusive end
                } else {
                    Ok(Range::Full)
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
        }
    }
}

fn parse_index(reader: &mut Reader) -> ParseResult<usize> {
    let position = reader.position();
    let index = parse_usize(reader)?;

    if index >= 1 {
        Ok(index - 1)
    } else {
        Err(ParseError {
            kind: ParseErrorKind::RangeIndexZero,
            start: position,
            end: reader.position(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start() {
        assert_eq!(Range::Full.start(), None);
        assert_eq!(Range::From(0).start(), Some(0));
        assert_eq!(Range::From(1).start(), Some(1));
        assert_eq!(Range::FromTo(0, 0).start(), Some(0));
        assert_eq!(Range::FromTo(0, 1).start(), Some(0));
        assert_eq!(Range::FromTo(1, 1).start(), Some(1));
        assert_eq!(Range::To(0).start(), None);
        assert_eq!(Range::To(1).start(), None);
    }

    #[test]
    fn length() {
        assert_eq!(Range::Full.start(), None);
        assert_eq!(Range::From(0).length(), None);
        assert_eq!(Range::FromTo(0, 0).length(), Some(0));
        assert_eq!(Range::FromTo(0, 1).length(), Some(1));
        assert_eq!(Range::FromTo(1, 1).length(), Some(0));
        assert_eq!(Range::To(0).length(), Some(0));
        assert_eq!(Range::To(1).length(), Some(1));
    }

    #[test]
    #[should_panic]
    fn length_invalid() {
        Range::FromTo(1, 0).length();
    }

    #[test]
    fn parse_empty_error() {
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
    fn parse_invalid_error() {
        let mut reader = Reader::from("a");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeInvalid("a".to_string()),
                start: 0,
                end: 1,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_full() {
        let mut reader = Reader::from("-");
        assert_eq!(Range::parse(&mut reader), Ok(Range::Full));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_start_zero_error() {
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
    fn parse_start() {
        let mut reader = Reader::from("1-");
        assert_eq!(Range::parse(&mut reader), Ok(Range::From(0)));
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn parse_end_zero_error() {
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
    fn parse_end() {
        let mut reader = Reader::from("-1");
        assert_eq!(Range::parse(&mut reader), Ok(Range::To(1)));
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn parse_start_over_end_error() {
        let mut reader = Reader::from("2-1");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::RangeStartOverEnd(2, 1),
                start: 0,
                end: 3,
            })
        );
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn parse_start_equals_end() {
        let mut reader = Reader::from("1-1");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(0, 1)));
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn parse_start_equals_end_short() {
        let mut reader = Reader::from("1");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(0, 1)));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_start_below_end() {
        let mut reader = Reader::from("1-2");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(0, 2)));
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn parse_ignore_remaining_chars() {
        let mut reader = Reader::from("1ab");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(0, 1)));
        assert_eq!(reader.position(), 1);
    }
}
