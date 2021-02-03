use crate::pattern::char::Char;
use crate::pattern::integer::{get_bits, parse_integer};
use crate::pattern::parse::Result;
use crate::pattern::range::{Range, RangeBound};
use crate::pattern::reader::Reader;
use rand::thread_rng;
use rand::Rng;

use std::fmt;

pub type Number = u64;

impl RangeBound for Number {
    const DELIMITER_REQUIRED: bool = true;
    const LENGTH_ALLOWED: bool = false;

    fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        parse_integer(reader)
    }

    fn unparse(self) -> String {
        self.to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct NumberInterval(Range<Number>);

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

    pub fn random(&self) -> Number {
        let start = self.start();
        let end = self.end().unwrap_or(Number::MAX);
        if start == 0 && end == Number::MAX {
            thread_rng().gen() // gen_range(start..=end) would cause an overflow in rand lib
        } else {
            thread_rng().gen_range(start..=end)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::parse::{Error, ErrorKind};

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
            assert_eq!(Number::unparse(123), String::from("123"));
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
        fn end() {
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
            fn start_with_length() {
                let mut reader = Reader::from("0+1");
                assert_eq!(
                    NumberInterval::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRangeDelimiter(Some(Char::Raw('+'))),
                        range: 1..2
                    })
                );
                assert_eq!(reader.position(), 2);
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

        mod random {
            use super::*;

            #[test]
            fn lowest() {
                assert_eq!(NumberInterval::new(0, Some(0)).random(), 0);
            }

            #[test]
            fn highest() {
                assert_eq!(NumberInterval::new(Number::MAX, None).random(), Number::MAX);
            }

            #[test]
            fn lowest_to_highest() {
                NumberInterval::new(0, Some(Number::MAX)).random(); // Should not overflow
                NumberInterval::new(1, Some(Number::MAX)).random();
                NumberInterval::new(0, Some(Number::MAX - 1)).random();
            }
        }
    }
}
