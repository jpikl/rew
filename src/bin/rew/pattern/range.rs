use crate::pattern::char::Char;
use crate::pattern::integer::{parse_integer, ParsableInt};
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::{LENGTH, RANGE};

pub trait RangeBound: ParsableInt {
    const DELIMITER_REQUIRED: bool;
    const LENGTH_ALLOWED: bool;

    fn parse(reader: &mut Reader<Char>) -> Result<Self>;

    fn unparse(self) -> String;
}

#[derive(Debug, PartialEq)]
pub struct Range<T: RangeBound>(pub T, pub Option<T>);

impl<T: RangeBound> Range<T> {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        match reader.peek_char() {
            Some('0'..='9') => {
                let position = reader.position();
                let start = T::parse(reader)?;

                match reader.peek_char() {
                    Some(RANGE) => {
                        reader.seek();

                        if let Some('0'..='9') = reader.peek_char() {
                            let end = T::parse(reader)?;
                            if start > end {
                                Err(Error {
                                    kind: ErrorKind::RangeStartOverEnd(
                                        start.unparse(),
                                        end.unparse(),
                                    ),
                                    range: position..reader.position(),
                                })
                            } else {
                                Ok(Range(start, Some(end)))
                            }
                        } else {
                            Ok(Range(start, None))
                        }
                    }

                    Some(LENGTH) if T::LENGTH_ALLOWED => {
                        reader.seek();

                        if let Some('0'..='9') = reader.peek_char() {
                            let position = reader.position();
                            let length: T = parse_integer(reader)?;
                            if let Some(end) = start.checked_add(&length) {
                                Ok(Range(start, Some(end)))
                            } else {
                                Err(Error {
                                    kind: ErrorKind::RangeLengthOverflow(
                                        length.to_string(),
                                        T::max_value().to_string(),
                                    ),
                                    range: position..reader.position(),
                                })
                            }
                        } else {
                            Err(Error {
                                kind: ErrorKind::ExpectedRangeLength,
                                range: reader.position()..reader.end(),
                            })
                        }
                    }

                    _ if T::DELIMITER_REQUIRED => {
                        let position = reader.position();
                        let char = reader.read();

                        Err(Error {
                            kind: ErrorKind::ExpectedRangeDelimiter(char.map(Clone::clone)),
                            range: position..reader.position(),
                        })
                    }

                    _ => Ok(Range(start, Some(start))),
                }
            }

            Some(_) => Err(Error {
                kind: ErrorKind::RangeInvalid(reader.peek_to_end().to_string()),
                range: reader.position()..reader.end(),
            }),

            None => Err(Error {
                kind: ErrorKind::ExpectedRange,
                range: reader.position()..reader.end(),
            }),
        }
    }
}
