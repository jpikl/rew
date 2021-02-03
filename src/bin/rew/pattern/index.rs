use std::fmt;

use crate::pattern::char::Char;
use crate::pattern::integer::parse_integer;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::range::{Range, RangeBound};
use crate::pattern::reader::Reader;

pub type Index = usize;

impl RangeBound for Index {
    const DELIMITER_REQUIRED: bool = false;
    const LENGTH_ALLOWED: bool = true;

    fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        let position = reader.position();
        let index: Index = parse_integer(reader)?;

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
pub struct IndexRange(Range<Index>);

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

    pub fn substr_backward(&self, mut value: String) -> String {
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
        match &self.0 {
            Range(start, Some(end)) => write!(formatter, "{}..{}", start + 1, end + 1),
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
            assert_eq!(Index::unparse(122), String::from("123"));
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
                    Ok(IndexRange::new(1, Some(4)))
                );
                assert_eq!(reader.position(), 3);
            }

            #[test]
            fn start_with_length_overflow() {
                let input = format!("3+{}", Index::max_value() - 1);
                let mut reader = Reader::from(input.as_str());
                assert_eq!(
                    IndexRange::parse(&mut reader),
                    Err(Error {
                        kind: ErrorKind::RangeLengthOverflow(
                            (Index::max_value() - 1).to_string(),
                            Index::max_value().to_string()
                        ),
                        range: 2..input.len()
                    })
                );
                assert_eq!(reader.position(), input.len());
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

        mod substr {
            use super::*;

            #[test]
            fn empty() {
                assert_eq!(
                    IndexRange::new(0, None).substr(String::new()),
                    String::new()
                );
            }

            #[test]
            fn from_first() {
                assert_eq!(
                    IndexRange::new(0, None).substr(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn from_last() {
                assert_eq!(
                    IndexRange::new(3, None).substr(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn from_over() {
                assert_eq!(
                    IndexRange::new(4, None).substr(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn from_first_to_first() {
                assert_eq!(
                    IndexRange::new(0, Some(0)).substr(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn from_first_to_last_but_one() {
                assert_eq!(
                    IndexRange::new(0, Some(2)).substr(String::from("ábčd")),
                    String::from("ábč")
                );
            }

            #[test]
            fn from_first_to_last() {
                assert_eq!(
                    IndexRange::new(0, Some(3)).substr(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn from_first_to_over() {
                assert_eq!(
                    IndexRange::new(0, Some(4)).substr(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn from_last_to_last() {
                assert_eq!(
                    IndexRange::new(3, Some(3)).substr(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn from_last_to_over() {
                assert_eq!(
                    IndexRange::new(3, Some(4)).substr(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn from_over_to_over() {
                assert_eq!(
                    IndexRange::new(4, Some(4)).substr(String::from("ábčd")),
                    String::new()
                );
            }
        }

        mod substr_backward {
            use super::*;

            #[test]
            fn empty() {
                assert_eq!(
                    IndexRange::new(0, None).substr_backward(String::new()),
                    String::new()
                );
            }

            #[test]
            fn from_first() {
                assert_eq!(
                    IndexRange::new(0, None).substr_backward(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn from_last() {
                assert_eq!(
                    IndexRange::new(3, None).substr_backward(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn from_over() {
                assert_eq!(
                    IndexRange::new(4, None).substr_backward(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn from_first_to_first() {
                assert_eq!(
                    IndexRange::new(0, Some(0)).substr_backward(String::from("ábčd")),
                    String::from("d")
                );
            }

            #[test]
            fn from_first_to_last_but_one() {
                assert_eq!(
                    IndexRange::new(0, Some(2)).substr_backward(String::from("ábčd")),
                    String::from("bčd")
                );
            }

            #[test]
            fn from_first_to_last() {
                assert_eq!(
                    IndexRange::new(0, Some(3)).substr_backward(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn from_first_to_over() {
                assert_eq!(
                    IndexRange::new(0, Some(4)).substr_backward(String::from("ábčd")),
                    String::from("ábčd")
                );
            }

            #[test]
            fn from_last_to_last() {
                assert_eq!(
                    IndexRange::new(3, Some(3)).substr_backward(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn from_last_to_over() {
                assert_eq!(
                    IndexRange::new(3, Some(3)).substr_backward(String::from("ábčd")),
                    String::from("á")
                );
            }

            #[test]
            fn from_over_to_over() {
                assert_eq!(
                    IndexRange::new(4, Some(4)).substr_backward(String::from("ábčd")),
                    String::new()
                );
            }

            #[test]
            fn from_extra_over_to_over() {
                // Covers different evaluation branch than from_over_to_over
                assert_eq!(
                    IndexRange::new(5, Some(5)).substr_backward(String::from("ábčd")),
                    String::new()
                );
            }
        }
    }
}
