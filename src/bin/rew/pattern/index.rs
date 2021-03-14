use std::fmt;

use crate::pattern::char::Char;
use crate::pattern::integer::parse_integer;
use crate::pattern::parse::{BaseResult, Error, ErrorKind, Result};
use crate::pattern::range::{Range, RangeType};
use crate::pattern::reader::Reader;

#[derive(PartialEq, Debug)]
pub struct Index;

pub type IndexRange = Range<Index>;
pub type IndexValue = usize;

impl RangeType for Index {
    const EMPTY_ALLOWED: bool = false;
    const DELIMITER_REQUIRED: bool = false;
    const LENGTH_DELIMITER_ALLOWED: bool = true;

    type Value = IndexValue;

    fn shift(value: Self::Value) -> BaseResult<Self::Value> {
        Index::shift(value)
    }
}

impl Index {
    pub fn parse(reader: &mut Reader<Char>) -> Result<IndexValue> {
        let position = reader.position();
        let index: IndexValue = parse_integer(reader)?;

        Self::shift(index).map_err(|kind| Error {
            kind,
            range: position..reader.position(),
        })
    }

    fn shift(index: IndexValue) -> BaseResult<IndexValue> {
        if index >= 1 {
            Ok(index - 1)
        } else {
            Err(ErrorKind::IndexZero)
        }
    }
}

impl IndexRange {
    pub fn substr(&self, mut value: String) -> String {
        if let Some((start, _)) = value.char_indices().nth(self.start()) {
            value.replace_range(..start, "");
        } else {
            value.clear();
        }

        if let Some(length) = self.length() {
            if let Some((end, _)) = value.char_indices().nth(length) {
                value.truncate(end);
            }
        }

        value
    }

    pub fn substr_back(&self, mut value: String) -> String {
        let start = self.start();
        if start > 0 {
            if let Some((start, _)) = value.char_indices().nth_back(start - 1) {
                value.truncate(start);
            } else {
                value.clear();
            }
        }

        if let Some(length) = self.length() {
            if length > 0 {
                if let Some((end, _)) = value.char_indices().nth_back(length - 1) {
                    value.replace_range(..end, "");
                }
            } else {
                value.clear();
            }
        }

        value
    }
}

impl fmt::Display for IndexRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Range(start, Some(end)) => write!(formatter, "{}..{}", start + 1, end),
            Range(start, None) => write!(formatter, "{}..", start + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ByteRange;

    mod index {
        use super::*;
        use test_case::test_case;

        #[test_case("abc", ErrorKind::ExpectedNumber, 0..3; "invalid")]
        #[test_case("0abc", ErrorKind::IndexZero, 0..1; "zero")]
        fn parse_err(input: &str, kind: ErrorKind, range: ByteRange) {
            assert_eq!(
                Index::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("1", 0; "one")]
        #[test_case("123abc", 122; "multiple digits and chars")]
        fn parse_ok(input: &str, result: IndexValue) {
            assert_eq!(Index::parse(&mut Reader::from(input)), Ok(result));
        }

        #[test_case(0, Err(ErrorKind::IndexZero); "zero")]
        #[test_case(1, Ok(0); "positive")]
        fn shift(index: IndexValue, result: BaseResult<IndexValue>) {
            assert_eq!(Index::shift(index), result)
        }
    }

    mod index_range {
        use super::*;
        use test_case::test_case;

        #[test_case(1, None, "2.."; "open")]
        #[test_case(1, Some(3), "2..3"; "closed")]
        fn display(start: IndexValue, end: Option<IndexValue>, result: &str) {
            assert_eq!(Range::<Index>(start, end).to_string(), result);
        }

        #[test_case("", ErrorKind::ExpectedRange, 0..0; "empty")]
        #[test_case("-", ErrorKind::RangeInvalid(String::from("-")), 0..1; "invalid")]
        #[test_case("0-", ErrorKind::IndexZero, 0..1; "zero start no end")]
        #[test_case("1-0", ErrorKind::IndexZero, 2..3; "start above zero end")]
        #[test_case("2-1", ErrorKind::RangeStartOverEnd(String::from("2"), String::from("1")), 0..3; "start above end")]
        #[test_case("1+", ErrorKind::ExpectedRangeLength, 2..2; "start no length")]
        #[test_case("1+ab", ErrorKind::ExpectedRangeLength, 2..4; "start no length but chars")]
        fn parse_err(input: &str, kind: ErrorKind, range: ByteRange) {
            assert_eq!(
                IndexRange::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("1", 0, Some(1); "no delimiter")]
        #[test_case("2ab", 1, Some(2); "no delimiter but chars")]
        #[test_case("1-", 0, None; "no end")]
        #[test_case("1-2", 0, Some(2); "start below end")]
        #[test_case("1-1", 0, Some(1); "start equals end")]
        #[test_case("2+0", 1, Some(1); "start with zero length")]
        #[test_case("2+3", 1, Some(4); "start with positive length")]
        #[test_case(&format!("2+{}", IndexValue::MAX), 1, None; "start with overflow length")]
        fn parse_ok(input: &str, start: IndexValue, end: Option<IndexValue>) {
            assert_eq!(
                IndexRange::parse(&mut Reader::from(input)),
                Ok(Range::<Index>(start, end))
            );
        }

        #[test_case("", 0, None, ""; "empty")]
        #[test_case("ábčd", 0, None, "ábčd"; "before first")]
        #[test_case("ábčd", 3, None, "d"; "before last")]
        #[test_case("ábčd", 4, None, ""; "after last")]
        #[test_case("ábčd", 5, None, ""; "over last")]
        #[test_case("ábčd", 0, Some(0), ""; "before first before first")]
        #[test_case("ábčd", 0, Some(1), "á"; "before first after first")]
        #[test_case("ábčd", 0, Some(3), "ábč"; "before first before last")]
        #[test_case("ábčd", 0, Some(4), "ábčd"; "before first after last")]
        #[test_case("ábčd", 0, Some(5), "ábčd"; "before first over last")]
        #[test_case("ábčd", 3, Some(3), ""; "before last before last")]
        #[test_case("ábčd", 3, Some(4), "d"; "before last after last")]
        #[test_case("ábčd", 3, Some(5), "d"; "before last over last")]
        #[test_case("ábčd", 4, Some(4), ""; "after last after last")]
        #[test_case("ábčd", 4, Some(5), ""; "after last over last")]
        #[test_case("ábčd", 5, Some(5), ""; "over last over last")]
        fn substr(input: &str, start: IndexValue, end: Option<IndexValue>, output: &str) {
            assert_eq!(
                Range::<Index>(start, end).substr(String::from(input)),
                output
            );
        }

        #[test_case("", 0, None, ""; "empty")]
        #[test_case("ábčd", 0, None, "ábčd"; "before first")]
        #[test_case("ábčd", 3, None, "á"; "before last")]
        #[test_case("ábčd", 4, None, ""; "after last")]
        #[test_case("ábčd", 5, None, ""; "over last")]
        #[test_case("ábčd", 0, Some(0), ""; "before first before first")]
        #[test_case("ábčd", 0, Some(1), "d"; "before first after first")]
        #[test_case("ábčd", 0, Some(3), "bčd"; "before first before last")]
        #[test_case("ábčd", 0, Some(4), "ábčd"; "before first after last")]
        #[test_case("ábčd", 0, Some(5), "ábčd"; "before first over last")]
        #[test_case("ábčd", 3, Some(3), ""; "before last before last")]
        #[test_case("ábčd", 3, Some(4), "á"; "before last after last")]
        #[test_case("ábčd", 3, Some(5), "á"; "before last over last")]
        #[test_case("ábčd", 4, Some(4), ""; "after last after last")]
        #[test_case("ábčd", 4, Some(5), ""; "after last over last")]
        #[test_case("ábčd", 5, Some(5), ""; "over last over last")]
        fn substr_back(input: &str, start: IndexValue, end: Option<IndexValue>, output: &str) {
            assert_eq!(
                Range::<Index>(start, end).substr_back(String::from(input)),
                output
            );
        }
    }
}
