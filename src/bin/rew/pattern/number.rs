use crate::pattern::integer::get_bits;
use crate::pattern::parse::ErrorKind;
use crate::pattern::range::{Range, RangeType};
use rand::thread_rng;
use rand::Rng;
use std::fmt;

#[derive(PartialEq, Debug)]
pub struct Number;

pub type NumberRange = Range<Number>;
pub type NumberValue = u64;

impl RangeType for Number {
    const EMPTY_ALLOWED: bool = true;
    const DELIMITER_REQUIRED: bool = true;
    const LENGTH_DELIMITER_ALLOWED: bool = false;

    type Value = NumberValue;

    fn shift(value: Self::Value) -> std::result::Result<Self::Value, ErrorKind> {
        Ok(value)
    }
}

impl NumberRange {
    pub fn random(&self) -> NumberValue {
        let start = self.start();
        let end = self.end().unwrap_or(NumberValue::MAX);

        if start == 0 && end == NumberValue::MAX {
            thread_rng().gen() // gen_range(start..=end) would cause an overflow in rand lib
        } else {
            thread_rng().gen_range(start..=end)
        }
    }
}

impl fmt::Display for NumberRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Range(start, Some(end)) => write!(formatter, "[{}, {}]", start, end),
            Range(start, None) => write!(formatter, "[{}, 2^{})", start, get_bits::<NumberValue>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::parse::{Error, ErrorKind};
    use crate::pattern::range::tests;

    #[test]
    fn start() {
        tests::start::<Number>()
    }

    #[test]
    fn end() {
        tests::end::<Number>()
    }

    #[test]
    fn length() {
        tests::length::<Number>()
    }

    #[test]
    fn display() {
        assert_eq!(Range::<Number>(1, None).to_string(), "[1, 2^64)");
        assert_eq!(Range::<Number>(1, Some(2)).to_string(), "[1, 2]");
    }

    mod parse {
        use super::*;
        use crate::pattern::char::Char;
        use crate::pattern::reader::Reader;

        #[test]
        fn empty() {
            let mut reader = Reader::from("");
            assert_eq!(
                NumberRange::parse(&mut reader),
                Ok(Range::<Number>(0, None))
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn invalid() {
            let mut reader = Reader::from("-");
            assert_eq!(
                NumberRange::parse(&mut reader),
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
                NumberRange::parse(&mut reader),
                Ok(Range::<Number>(0, None))
            );
            assert_eq!(reader.position(), 2);
        }

        #[test]
        fn start_below_end() {
            let mut reader = Reader::from("0-1");
            assert_eq!(
                NumberRange::parse(&mut reader),
                Ok(Range::<Number>(0, Some(2)))
            );
            assert_eq!(reader.position(), 3);
        }

        #[test]
        fn start_equals_end() {
            let mut reader = Reader::from("0-0");
            assert_eq!(
                NumberRange::parse(&mut reader),
                Ok(Range::<Number>(0, Some(1)))
            );
            assert_eq!(reader.position(), 3);
        }

        #[test]
        fn start_above_end() {
            let mut reader = Reader::from("1-0");
            assert_eq!(
                NumberRange::parse(&mut reader),
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
                NumberRange::parse(&mut reader),
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
                NumberRange::parse(&mut reader),
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
                NumberRange::parse(&mut reader),
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
            assert_eq!(Range::<Number>(0, Some(0)).random(), 0);
        }

        #[test]
        fn highest() {
            assert_eq!(
                Range::<Number>(NumberValue::MAX, None).random(),
                NumberValue::MAX
            );
        }

        #[test]
        fn lowest_to_highest() {
            Range::<Number>(0, Some(NumberValue::MAX)).random(); // Should not overflow
            Range::<Number>(1, Some(NumberValue::MAX)).random();
            Range::<Number>(0, Some(NumberValue::MAX - 1)).random();
        }
    }
}
