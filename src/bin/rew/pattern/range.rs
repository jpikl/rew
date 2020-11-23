use crate::pattern::char::Char;
use crate::pattern::number::{get_bits, parse_number};
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::RANGE;
use num_traits::PrimInt;
use std::fmt;

pub type Number = u64;
pub type Index = usize;

pub trait RangeBound: PrimInt {
    const REQUIRES_DELIMTER: bool;

    fn parse(reader: &mut Reader<Char>) -> Result<Self>;

    fn unparse(self) -> String;
}

impl RangeBound for Number {
    const REQUIRES_DELIMTER: bool = true;

    fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        parse_number(reader)
    }

    fn unparse(self) -> String {
        self.to_string()
    }
}

impl RangeBound for Index {
    const REQUIRES_DELIMTER: bool = false;

    fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        let position = reader.position();
        let index: Index = parse_number(reader)?;

        if index >= 1 {
            Ok(index - 1)
        } else {
            Err(Error {
                kind: ErrorKind::RangeIndexZero,
                range: position..reader.position(),
            })
        }
    }

    fn unparse(self) -> String {
        (self + 1).to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct NumberInterval(Range<Number>);

#[derive(Debug, PartialEq)]
pub struct IndexRange(Range<Index>);

#[derive(Debug, PartialEq)]
pub struct Range<T: RangeBound>(T, Option<T>);

impl NumberInterval {
    #[cfg(test)]
    pub fn new(start: Number, end: Option<Number>) -> Self {
        Self(Range(start, end))
    }

    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        if reader.peek().is_some() {
            Range::parse(reader).map(Self)
        } else {
            Ok(Self(Range(0, None)))
        }
    }

    pub fn start(&self) -> Number {
        (self.0).0
    }

    pub fn end(&self) -> Option<Number> {
        (self.0).1
    }
}

impl IndexRange {
    #[cfg(test)]
    pub fn new(start: Index, end: Option<Index>) -> Self {
        Self(Range(start, end))
    }

    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        Range::parse(reader).map(Self)
    }

    pub fn start(&self) -> Index {
        (self.0).0
    }

    pub fn length(&self) -> Option<Index> {
        (self.0).1.map(|end| {
            let start = self.start();
            assert!(end >= start, "IndexRange start {} > end {}", start, end);
            end - start + 1
        })
    }
}

impl fmt::Display for NumberInterval {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Range(start, Some(end)) => write!(formatter, "[{}, {}]", start, end),
            Range(start, None) => write!(formatter, "[{}, 2^{})", start, get_bits::<Number>()),
        }
    }
}

impl fmt::Display for IndexRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Range(start, Some(end)) => write!(formatter, "{}..{}", start + 1, end + 1),
            Range(start, None) => write!(formatter, "{}..", start + 1),
        }
    }
}

impl<T: RangeBound> Range<T> {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        match reader.peek_char() {
            Some('0'..='9') => {
                let position = reader.position();
                let start = T::parse(reader)?;

                if let Some(RANGE) = reader.peek_char() {
                    reader.seek();

                    if let Some('0'..='9') = reader.peek_char() {
                        let end = T::parse(reader)?;
                        if start > end {
                            Err(Error {
                                kind: ErrorKind::RangeStartOverEnd(start.unparse(), end.unparse()),
                                range: position..reader.position(),
                            })
                        } else {
                            Ok(Range(start, Some(end)))
                        }
                    } else {
                        Ok(Range(start, None))
                    }
                } else if T::REQUIRES_DELIMTER {
                    let position = reader.position();
                    let char = reader.read();

                    Err(Error {
                        kind: ErrorKind::ExpectedRangeDelimiter(char.map(Clone::clone)),
                        range: position..reader.position(),
                    })
                } else {
                    Ok(Range(start, Some(start)))
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

#[cfg(test)]
mod tests {
    use super::*;

    mod number {
        use super::*;

        mod parse {
            use super::*;

            #[test]
            fn zero() {
                let mut reader = Reader::from("0abc");
                assert_eq!(Number::parse(&mut reader), Ok(0));
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn positive() {
                let mut reader = Reader::from("123abc");
                assert_eq!(Number::parse(&mut reader), Ok(123));
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn invalid() {
                let mut reader = Reader::from("abc");
                assert_eq!(
                    Number::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedNumber,
                        range: 0..3
                    })
                );
                assert_eq!(reader.position(), 0);
            }
        }

        #[test]
        fn unparse() {
            assert_eq!((123 as Number).unparse(), String::from("123"));
        }
    }

    mod index {
        use super::*;

        mod parse {
            use super::*;

            #[test]
            fn zero() {
                let mut reader = Reader::from("0abc");
                assert_eq!(
                    Index::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeIndexZero,
                        range: 0..1
                    })
                );
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn positive() {
                let mut reader = Reader::from("123abc");
                assert_eq!(Index::parse(&mut reader), Ok(122));
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn invalid() {
                let mut reader = Reader::from("abc");
                assert_eq!(
                    Index::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedNumber,
                        range: 0..3
                    })
                );
                assert_eq!(reader.position(), 0);
            }
        }

        #[test]
        fn unparse() {
            assert_eq!((122 as Index).unparse(), String::from("123"));
        }
    }

    mod number_interval {
        use super::*;

        #[test]
        fn start() {
            assert_eq!(NumberInterval::new(0, None).start(), 0);
            assert_eq!(NumberInterval::new(1, None).start(), 1);
        }

        #[test]
        fn length() {
            assert_eq!(NumberInterval::new(0, None).end(), None);
            assert_eq!(NumberInterval::new(0, Some(0)).end(), Some(0));
            assert_eq!(NumberInterval::new(0, Some(1)).end(), Some(1));
        }

        #[test]
        fn display() {
            assert_eq!(NumberInterval::new(1, None).to_string(), "[1, 2^64)");
            assert_eq!(NumberInterval::new(1, Some(2)).to_string(), "[1, 2]");
        }

        mod parse {
            use super::*;

            #[test]
            fn empty() {
                let mut reader = Reader::from("");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Ok(NumberInterval::new(0, None))
                );
                assert_eq!(reader.position(), 0);
            }

            #[test]
            fn invalid() {
                let mut reader = Reader::from("-");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeInvalid(String::from("-")),
                        range: 0..1,
                    })
                );
                assert_eq!(reader.position(), 0);
            }

            #[test]
            fn start_no_end() {
                let mut reader = Reader::from("0-");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Ok(NumberInterval::new(0, None))
                );
                assert_eq!(reader.position(), 2);
            }

            #[test]
            fn start_below_end() {
                let mut reader = Reader::from("0-1");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Ok(NumberInterval::new(0, Some(1)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_equals_end() {
                let mut reader = Reader::from("0-0");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Ok(NumberInterval::new(0, Some(0)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_above_end() {
                let mut reader = Reader::from("1-0");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeStartOverEnd(String::from("1"), String::from("0")),
                        range: 0..3,
                    })
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn no_delimiter() {
                let mut reader = Reader::from("1");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRangeDelimiter(None),
                        range: 1..1
                    })
                );
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn no_delimiter_but_chars() {
                let mut reader = Reader::from("1ab");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRangeDelimiter(Some(Char::Raw('a'))),
                        range: 1..2
                    })
                );
                assert_eq!(reader.position(), 2);
            }
        }
    }

    mod index_range {
        use super::*;

        #[test]
        fn start() {
            assert_eq!(IndexRange::new(0, None).start(), 0);
            assert_eq!(IndexRange::new(1, None).start(), 1);
        }

        #[test]
        fn length() {
            assert_eq!(IndexRange::new(0, None).length(), None);
            assert_eq!(IndexRange::new(1, None).length(), None);
            assert_eq!(IndexRange::new(0, Some(0)).length(), Some(1));
            assert_eq!(IndexRange::new(0, Some(1)).length(), Some(2));
            assert_eq!(IndexRange::new(1, Some(1)).length(), Some(1));
        }

        #[test]
        fn display() {
            assert_eq!(IndexRange::new(1, None).to_string(), "2..");
            assert_eq!(IndexRange::new(1, Some(2)).to_string(), "2..3");
        }

        mod parse {
            use super::*;

            #[test]
            fn empty() {
                let mut reader = Reader::from("");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRange,
                        range: 0..0,
                    })
                );
                assert_eq!(reader.position(), 0);
            }

            #[test]
            fn invalid() {
                let mut reader = Reader::from("-");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeInvalid(String::from("-")),
                        range: 0..1,
                    })
                );
                assert_eq!(reader.position(), 0);
            }

            #[test]
            fn zero_start_no_end() {
                let mut reader = Reader::from("0-");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeIndexZero,
                        range: 0..1,
                    })
                );
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn start_no_end() {
                let mut reader = Reader::from("1-");
                assert_eq!(IndexRange::parse(&mut reader), Ok(IndexRange::new(0, None)));
                assert_eq!(reader.position(), 2);
            }

            #[test]
            fn start_above_zero_end() {
                let mut reader = Reader::from("1-0");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeIndexZero,
                        range: 2..3,
                    })
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_below_end() {
                let mut reader = Reader::from("1-2");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(IndexRange::new(0, Some(1)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_equals_end() {
                let mut reader = Reader::from("1-1");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(IndexRange::new(0, Some(0)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_above_end() {
                let mut reader = Reader::from("2-1");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeStartOverEnd(String::from("2"), String::from("1")),
                        range: 0..3,
                    })
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn no_delimiter() {
                let mut reader = Reader::from("1");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(IndexRange::new(0, Some(0)))
                );
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn no_delimiter_but_chars() {
                let mut reader = Reader::from("1ab");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(IndexRange::new(0, Some(0)))
                );
                assert_eq!(reader.position(), 1);
            }
        }
    }
}
