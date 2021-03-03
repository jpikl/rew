use crate::pattern::char::Char;
use crate::pattern::integer::{parse_integer, ParsableInt};
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::{LENGTH, RANGE};
use num_traits::{CheckedAdd, One, Zero};

pub trait RangeType {
    const EMPTY_ALLOWED: bool;
    const DELIMITER_REQUIRED: bool;
    const LENGTH_DELIMITER_ALLOWED: bool;

    type Value: ParsableInt;

    fn shift(value: Self::Value) -> std::result::Result<Self::Value, ErrorKind>;
}

#[derive(Debug, PartialEq)]
pub struct Range<T: RangeType>(pub T::Value, pub Option<T::Value>);

impl<T: RangeType> Range<T> {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        match reader.peek_char() {
            Some('0'..='9') => {
                let start_pos = reader.position();
                let start_val = parse_integer(reader)?;

                let start = T::shift(start_val).map_err(|kind| Error {
                    kind,
                    range: start_pos..reader.position(),
                })?;

                match reader.peek_char() {
                    Some(RANGE) => {
                        reader.seek();

                        if let Some('0'..='9') = reader.peek_char() {
                            let end_pos = reader.position();
                            let end_val = parse_integer(reader)?;

                            let end = T::shift(end_val).map_err(|kind| Error {
                                kind,
                                range: end_pos..reader.position(),
                            })?;

                            if start_val > end_val {
                                Err(Error {
                                    kind: ErrorKind::RangeStartOverEnd(
                                        start_val.to_string(),
                                        end_val.to_string(),
                                    ),
                                    range: start_pos..reader.position(),
                                })
                            } else if let Some(end) = end.checked_add(&T::Value::one()) {
                                Ok(Self(start, Some(end)))
                            } else {
                                Ok(Self(start, None))
                            }
                        } else {
                            Ok(Self(start, None))
                        }
                    }

                    Some(LENGTH) if T::LENGTH_DELIMITER_ALLOWED => {
                        reader.seek();

                        if let Some('0'..='9') = reader.peek_char() {
                            let length: T::Value = parse_integer(reader)?;

                            if let Some(end) = start.checked_add(&length) {
                                Ok(Self(start, Some(end)))
                            } else {
                                Ok(Self(start, None))
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

                    _ => Ok(Self(start, start.checked_add(&T::Value::one()))),
                }
            }

            Some(_) => Err(Error {
                kind: ErrorKind::RangeInvalid(reader.peek_to_end().to_string()),
                range: reader.position()..reader.end(),
            }),

            None if T::EMPTY_ALLOWED => Ok(Self(T::Value::zero(), None)),

            None => Err(Error {
                kind: ErrorKind::ExpectedRange,
                range: reader.position()..reader.end(),
            }),
        }
    }

    pub fn start(&self) -> T::Value {
        self.0
    }

    pub fn end(&self) -> Option<T::Value> {
        self.1
    }

    pub fn length(&self) -> Option<T::Value> {
        self.1.map(|end| {
            let start = self.start();
            assert!(end >= start, "IndexRange start {} > end {}", start, end);
            end - start
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use num_traits::{One, Zero};

    pub fn start<T: RangeType>() {
        let v_0 = T::Value::zero();
        let v_1 = T::Value::one();

        assert_eq!(Range::<T>(v_0, None).start(), v_0);
        assert_eq!(Range::<T>(v_1, None).start(), v_1);
    }

    pub fn end<T: RangeType>() {
        let v_0 = T::Value::zero();
        let v_1 = T::Value::one();

        assert_eq!(Range::<T>(v_0, None).end(), None);
        assert_eq!(Range::<T>(v_0, Some(v_0)).end(), Some(v_0));
        assert_eq!(Range::<T>(v_0, Some(v_1)).end(), Some(v_1));
    }

    pub fn length<T: RangeType>() {
        let v_0 = T::Value::zero();
        let v_1 = T::Value::one();

        assert_eq!(Range::<T>(v_0, None).length(), None);
        assert_eq!(Range::<T>(v_1, None).length(), None);
        assert_eq!(Range::<T>(v_0, Some(v_0)).length(), Some(v_0));
        assert_eq!(Range::<T>(v_0, Some(v_1)).length(), Some(v_1));
        assert_eq!(Range::<T>(v_1, Some(v_1)).length(), Some(v_0));
    }
}
