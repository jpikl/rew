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
            Range(start, Some(end)) => write!(formatter, "[{}, {}]", start, end),
            Range(start, None) => write!(formatter, "[{}, 2^{})", start, get_bits::<NumberValue>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::pattern::char::Char;
    use crate::pattern::parse::{Error, ErrorKind};
    use crate::pattern::reader::Reader;

    use super::*;

    #[test_case(1, None, "[1, 2^64)"; "open")]
    #[test_case(1, Some(2), "[1, 2]"; "closed")]
    fn display(start: NumberValue, end: Option<NumberValue>, result: &str) {
        assert_eq!(Range::<Number>(start, end).to_string(), result);
    }

    #[test_case("-", ErrorKind::RangeInvalid(String::from("-")), 0..1; "invalid")]
    #[test_case("2-1", ErrorKind::RangeStartOverEnd(String::from("2"), String::from("1")), 0..3; "start above end")]
    #[test_case("0+1", ErrorKind::ExpectedRangeDelimiter(Some(Char::Raw('+'))), 1..2; "start with length")]
    #[test_case("1", ErrorKind::ExpectedRangeDelimiter(None), 1..1; "no delimiter")]
    #[test_case("1ab", ErrorKind::ExpectedRangeDelimiter(Some(Char::Raw('a'))), 1..2; "wrong delimiter")]
    fn parse_err(input: &str, kind: ErrorKind, range: std::ops::Range<usize>) {
        assert_eq!(
            NumberRange::parse(&mut Reader::from(input)),
            Err(Error { kind, range })
        );
    }

    #[test_case("", 0, None; "empty")]
    #[test_case("0-", 0, None; "start no end")]
    #[test_case("0-1", 0, Some(2); "start below end")]
    #[test_case("1-1", 1, Some(2); "start equals end")]
    fn parse_ok(input: &str, start: NumberValue, end: Option<NumberValue>) {
        assert_eq!(
            NumberRange::parse(&mut Reader::from(input)),
            Ok(Range::<Number>(start, end))
        );
    }

    #[test_case(0, Some(0), 0; "lowest")]
    #[test_case(NumberValue::MAX, None, NumberValue::MAX; "highest")]
    fn random_certain(start: NumberValue, end: Option<NumberValue>, result: NumberValue) {
        assert_eq!(Range::<Number>(start, end).random(), result);
    }

    #[test_case(0, Some(NumberValue::MAX); "from 0 to max")] // Should not overflow
    #[test_case(1, Some(NumberValue::MAX); "from 1 to max")]
    #[test_case(0, Some(NumberValue::MAX - 1); "from 0 to max-1")]
    fn random_uncertain(start: NumberValue, end: Option<NumberValue>) {
        Range::<Number>(start, end).random();
    }
}
