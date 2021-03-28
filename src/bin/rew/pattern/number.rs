use crate::pattern::integer::get_bits;
use crate::pattern::parse::BaseResult;
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

    fn shift(value: Self::Value) -> BaseResult<Self::Value> {
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
            Range(start, Some(end)) => write!(formatter, "[{}, {}]", start, end - 1),
            Range(start, None) => write!(formatter, "[{}, 2^{})", start, get_bits::<NumberValue>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    mod parse {
        use super::*;
        use crate::pattern::parse::{Error, ErrorKind};
        use crate::pattern::reader::Reader;
        use crate::utils::ByteRange;
        use test_case::test_case;

        #[test_case("-",   0..1, ErrorKind::RangeInvalid("-".into())                  ; "invalid")]
        #[test_case("2-1", 0..3, ErrorKind::RangeStartOverEnd("2".into(), "1".into()) ; "start above end")]
        #[test_case("0+1", 1..2, ErrorKind::ExpectedRangeDelimiter(Some('+'.into()))  ; "start with length")]
        #[test_case("1",   1..1, ErrorKind::ExpectedRangeDelimiter(None)              ; "no delimiter")]
        #[test_case("1ab", 1..2, ErrorKind::ExpectedRangeDelimiter(Some('a'.into()))  ; "wrong delimiter")]
        fn err(input: &str, range: ByteRange, kind: ErrorKind) {
            assert_eq!(
                NumberRange::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("",    0, None    ; "empty")]
        #[test_case("0-",  0, None    ; "start no end")]
        #[test_case("0-1", 0, Some(2) ; "start below end")]
        #[test_case("1-1", 1, Some(2) ; "start equals end")]
        fn ok(input: &str, start: NumberValue, end: Option<NumberValue>) {
            assert_eq!(
                NumberRange::parse(&mut Reader::from(input)),
                Ok(Range::<Number>(start, end))
            );
        }
    }

    mod random {
        use super::*;
        use test_case::test_case;

        const MAX: NumberValue = NumberValue::MAX;

        #[test_case(0,   Some(0), 0   ; "lowest")]
        #[test_case(MAX, None,    MAX ; "highest")]
        fn certain(start: NumberValue, end: Option<NumberValue>, result: NumberValue) {
            assert_eq!(Range::<Number>(start, end).random(), result);
        }

        #[test_case(0, Some(MAX)     ; "from 0 to max")] // Should not overflow
        #[test_case(1, Some(MAX)     ; "from 1 to max")]
        #[test_case(0, Some(MAX - 1) ; "from 0 to max-1")]
        fn uncertain(start: NumberValue, end: Option<NumberValue>) {
            Range::<Number>(start, end).random();
        }
    }

    #[test_case(1, None,    "[1, 2^64)" ; "open")]
    #[test_case(1, Some(3), "[1, 2]"    ; "closed")]
    fn display(start: NumberValue, end: Option<NumberValue>, result: &str) {
        assert_eq!(Range::<Number>(start, end).to_string(), result);
    }
}
