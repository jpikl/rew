use crate::pattern::char::Char;
use crate::pattern::number::parse_usize;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::RANGE;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Range {
    From(usize),
    FromTo(usize, usize),
    To(usize),
}

impl Range {
    pub fn start(&self) -> Option<usize> {
        match self {
            Self::From(start) => Some(*start),
            Self::FromTo(start, _) => Some(*start),
            Self::To(_) => None,
        }
    }

    pub fn length(&self) -> Option<usize> {
        match self {
            Self::From(_) => None,
            Self::FromTo(start, end) => {
                if start > end {
                    panic!("Range start ({}) > end ({})", start, end)
                }
                Some(end - start)
            }
            Self::To(end) => Some(*end),
        }
    }

    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        // For users, indices start from 1, end is inclusive.
        // Internally, indices start from 0, end is exclusive.

        match reader.peek_char() {
            Some('0'..='9') => {
                let position = reader.position();
                let start = parse_index(reader)?;

                if let Some(RANGE) = reader.peek_char() {
                    reader.seek();

                    if let Some('0'..='9') = reader.peek_char() {
                        let end = parse_index(reader)?;
                        if start > end {
                            Err(Error {
                                kind: ErrorKind::RangeStartOverEnd(start + 1, end + 1),
                                range: position..reader.position(),
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

            Some(RANGE) => {
                reader.seek();

                if let Some('0'..='9') = reader.peek_char() {
                    let end = parse_index(reader)?;
                    Ok(Range::To(end + 1)) // Inclusive end -> exclusive end
                } else {
                    Err(Error {
                        kind: ErrorKind::RangeUnbounded,
                        range: (reader.position() - 1)..reader.position(),
                    })
                }
            }

            Some(_) => Err(Error {
                kind: ErrorKind::RangeInvalid(Char::join(reader.peek_to_end())),
                range: reader.position()..reader.end(),
            }),

            None => Err(Error {
                kind: ErrorKind::ExpectedRange,
                range: reader.position()..reader.end(),
            }),
        }
    }
}

fn parse_index(reader: &mut Reader<Char>) -> Result<usize> {
    let position = reader.position();
    let index = parse_usize(reader)?;

    if index >= 1 {
        Ok(index - 1)
    } else {
        Err(Error {
            kind: ErrorKind::RangeIndexZero,
            range: position..reader.position(),
        })
    }
}

impl fmt::Display for Range {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // For users, indices start from 1, end is inclusive.
            // Internally, indices start from 0, end is exclusive.
            Self::From(start) => write!(formatter, "from {} to end", start + 1),
            Self::FromTo(start, end) => write!(formatter, "from {} to {}", start + 1, end),
            Self::To(end) => write!(formatter, "from start to {}", end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start() {
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
            Err(Error {
                kind: ErrorKind::ExpectedRange,
                range: 0..0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_unbounded_error() {
        let mut reader = Reader::from("-");
        assert_eq!(
            Range::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::RangeUnbounded,
                range: 0..1,
            })
        );
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_invalid_error() {
        let mut reader = Reader::from("a");
        assert_eq!(
            Range::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::RangeInvalid(String::from("a")),
                range: 0..1,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_start_zero_error() {
        let mut reader = Reader::from("0-");
        assert_eq!(
            Range::parse(&mut reader),
            Err(Error {
                kind: ErrorKind::RangeIndexZero,
                range: 0..1,
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
            Err(Error {
                kind: ErrorKind::RangeIndexZero,
                range: 1..2,
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
            Err(Error {
                kind: ErrorKind::RangeStartOverEnd(2, 1),
                range: 0..3,
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

    #[test]
    fn fmt() {
        // For users, indices start from 1, end is inclusive.
        // Internally, indices start from 0, end is exclusive.
        assert_eq!(Range::From(1).to_string(), "from 2 to end");
        assert_eq!(Range::FromTo(1, 3).to_string(), "from 2 to 3");
        assert_eq!(Range::To(3).to_string(), "from start to 3");
    }
}
