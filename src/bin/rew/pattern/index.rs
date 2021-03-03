use std::fmt;

use crate::pattern::char::Char;
use crate::pattern::integer::parse_integer;
use crate::pattern::parse::{Error, ErrorKind, Result};
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

    fn shift(value: Self::Value) -> std::result::Result<Self::Value, ErrorKind> {
        Index::shift(value)
    }
}

impl Index {
    #[allow(dead_code)] // Only temporary ... it's going to be used by future development
    fn parse(reader: &mut Reader<Char>) -> Result<IndexValue> {
        let position = reader.position();
        let index: IndexValue = parse_integer(reader)?;

        Self::shift(index).map_err(|kind| Error {
            kind,
            range: position..reader.position(),
        })
    }

    fn shift(index: IndexValue) -> std::result::Result<IndexValue, ErrorKind> {
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
                        kind: ErrorKind::IndexZero,
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

        mod shift {
            use super::*;

            #[test]
            fn valid() {
                assert_eq!(Index::shift(1), Ok(0));
            }

            #[test]
            fn invalid() {
                assert_eq!(Index::shift(0), Err(ErrorKind::IndexZero));
            }
        }
    }

    mod index_range {
        use super::*;
        use crate::pattern::range::tests;

        #[test]
        fn start() {
            tests::start::<Index>()
        }

        #[test]
        fn end() {
            tests::end::<Index>()
        }

        #[test]
        fn length() {
            tests::length::<Index>()
        }

        #[test]
        fn display() {
            assert_eq!(Range::<Index>(1, None).to_string(), "2..");
            assert_eq!(Range::<Index>(1, Some(3)).to_string(), "2..3");
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
                        kind: ErrorKind::IndexZero,
                        range: 0..1,
                    })
                );
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn start_no_end() {
                let mut reader = Reader::from("1-");
                assert_eq!(IndexRange::parse(&mut reader), Ok(Range::<Index>(0, None)));
                assert_eq!(reader.position(), 2);
            }

            #[test]
            fn start_above_zero_end() {
                let mut reader = Reader::from("1-0");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::IndexZero,
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
                    Ok(Range::<Index>(0, Some(2)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_equals_end() {
                let mut reader = Reader::from("1-1");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(Range::<Index>(0, Some(1)))
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
            fn start_no_length() {
                let mut reader = Reader::from("1+");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRangeLength,
                        range: 2..2
                    })
                );
                assert_eq!(reader.position(), 2);
            }

            #[test]
            fn start_no_length_but_chars() {
                let mut reader = Reader::from("1+ab");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::ExpectedRangeLength,
                        range: 2..4
                    })
                );
                assert_eq!(reader.position(), 2);
            }

            #[test]
            fn start_with_length() {
                let mut reader = Reader::from("2+3");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(Range::<Index>(1, Some(4)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_with_length_overflow() {
                let input = format!("2+{}", IndexValue::MAX);
                let mut reader = Reader::from(input.as_str());
                assert_eq!(IndexRange::parse(&mut reader), Ok(Range::<Index>(1, None)));
                assert_eq!(reader.position(), input.len());
            }

            #[test]
            fn no_delimiter() {
                let mut reader = Reader::from("1");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(Range::<Index>(0, Some(1)))
                );
                assert_eq!(reader.position(), 1);
            }

            #[test]
            fn no_delimiter_but_chars() {
                let mut reader = Reader::from("1ab");
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Ok(Range::<Index>(0, Some(1)))
                );
                assert_eq!(reader.position(), 1);
            }
        }

        mod substr {
            use super::*;

            #[test]
            fn empty() {
                assert_eq!(Range::<Index>(0, None).substr(String::new()), String::new());
            }

            #[test]
            fn before_first() {
                assert_eq!(
                    Range::<Index>(0, None).substr(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn before_last() {
                assert_eq!(
                    Range::<Index>(3, None).substr(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn after_last() {
                assert_eq!(
                    Range::<Index>(4, None).substr(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn before_first_before_first() {
                assert_eq!(
                    Range::<Index>(0, Some(0)).substr(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn before_first_after_first() {
                assert_eq!(
                    Range::<Index>(0, Some(1)).substr(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn before_first_before_last() {
                assert_eq!(
                    Range::<Index>(0, Some(3)).substr(String::from("ábčd")),
                    String::from("ábč")
                );
            }

            #[test]
            fn before_first_after_last() {
                assert_eq!(
                    Range::<Index>(0, Some(4)).substr(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn before_first_over_last() {
                assert_eq!(
                    Range::<Index>(0, Some(5)).substr(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn before_last_after_last() {
                assert_eq!(
                    Range::<Index>(3, Some(4)).substr(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn before_last_over_last() {
                assert_eq!(
                    Range::<Index>(3, Some(5)).substr(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn after_last_over_last() {
                assert_eq!(
                    Range::<Index>(4, Some(5)).substr(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn over_last_over_last() {
                assert_eq!(
                    Range::<Index>(5, Some(5)).substr(String::from("ábčd")),
                    String::new()
                );
            }
        }

        mod substr_back {
            use super::*;

            #[test]
            fn empty() {
                assert_eq!(
                    Range::<Index>(0, None).substr_back(String::new()),
                    String::new()
                );
            }

            #[test]
            fn before_first() {
                assert_eq!(
                    Range::<Index>(0, None).substr_back(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn before_last() {
                assert_eq!(
                    Range::<Index>(3, None).substr_back(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn after_last() {
                assert_eq!(
                    Range::<Index>(4, None).substr_back(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn before_first_before_first() {
                assert_eq!(
                    Range::<Index>(0, Some(0)).substr_back(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn before_first_after_first() {
                assert_eq!(
                    Range::<Index>(0, Some(1)).substr_back(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn before_first_before_last() {
                assert_eq!(
                    Range::<Index>(0, Some(3)).substr_back(String::from("ábčd")),
                    String::from("bčd")
                );
            }

            #[test]
            fn before_first_after_last() {
                assert_eq!(
                    Range::<Index>(0, Some(4)).substr_back(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn before_first_over_last() {
                assert_eq!(
                    Range::<Index>(0, Some(5)).substr_back(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn before_last_after_last() {
                assert_eq!(
                    Range::<Index>(3, Some(4)).substr_back(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn before_last_over_last() {
                assert_eq!(
                    Range::<Index>(3, Some(5)).substr_back(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn after_last_over_last() {
                assert_eq!(
                    Range::<Index>(4, Some(5)).substr_back(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn over_last_over_last() {
                assert_eq!(
                    Range::<Index>(5, Some(5)).substr_back(String::from("ábčd")),
                    String::new()
                );
            }
        }
    }
}
