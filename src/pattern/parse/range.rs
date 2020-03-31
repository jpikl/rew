use crate::pattern::parse::number::parse_usize;
use crate::pattern::parse::reader::Reader;
use crate::pattern::parse::types::ParseError;
use crate::pattern::types::Range;

const DIVIDER: char = '-';

impl Range {
    pub fn parse(reader: &mut Reader) -> Result<Self, ParseError> {
        match reader.peek() {
            Some('0'..='9') => {
                let offset = parse_offset(reader)?;
                if let Some(DIVIDER) = reader.peek() {
                    reader.read();
                    if let Some('0'..='9') = reader.peek() {
                        let length = parse_length(reader, offset)?;
                        Ok(Range { offset, length })
                    } else {
                        Ok(Range { offset, length: 0 })
                    }
                } else {
                    Ok(Range { offset, length: 1 })
                }
            }
            Some(DIVIDER) => {
                reader.read();
                if let Some('0'..='9') = reader.peek() {
                    let length = parse_length(reader, 0)?;
                    Ok(Range { offset: 0, length })
                } else {
                    Ok(Range {
                        offset: 0,
                        length: 0,
                    })
                }
            }
            _ => Err(ParseError {
                message: "Expected range",
                position: reader.posistion(),
            }),
        }
    }
}

fn parse_offset(reader: &mut Reader) -> Result<usize, ParseError> {
    let position = reader.posistion();
    let index = parse_usize(reader)?;

    if index < 1 {
        Err(ParseError {
            message: "Range indices starts from 1",
            position,
        })
    } else {
        Ok(index - 1)
    }
}

fn parse_length(reader: &mut Reader, offset: usize) -> Result<usize, ParseError> {
    let position = reader.posistion();
    let index = parse_usize(reader)?;

    if index < 1 {
        Err(ParseError {
            message: "Range indices starts from 1",
            position,
        })
    } else if index <= offset {
        Err(ParseError {
            message: "Range end cannot precede start",
            position,
        })
    } else {
        Ok(index - offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_as_error() {
        let mut reader = Reader::new("");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                message: "Expected range",
                position: 0,
            })
        );
        assert_eq!(reader.posistion(), 0);
    }

    #[test]
    fn parse_non_range_as_error() {
        let mut reader = Reader::new("a");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                message: "Expected range",
                position: 0,
            })
        );
        assert_eq!(reader.posistion(), 0);
    }

    #[test]
    fn parse_full() {
        let mut reader = Reader::new("-");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 0
            })
        );
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_full_ignore_rest() {
        let mut reader = Reader::new("-a");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 0
            })
        );
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_zero_start_as_error() {
        let mut reader = Reader::new("0-");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                message: "Range indices starts from 1",
                position: 0,
            })
        );
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_start() {
        let mut reader = Reader::new("1-");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 0
            })
        );
        assert_eq!(reader.posistion(), 2);
    }

    #[test]
    fn parse_start_ignore_rest() {
        let mut reader = Reader::new("10-a");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 9,
                length: 0
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_zero_end_as_error() {
        let mut reader = Reader::new("-0");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                message: "Range indices starts from 1",
                position: 1,
            })
        );
        assert_eq!(reader.posistion(), 2);
    }

    #[test]
    fn parse_end() {
        let mut reader = Reader::new("-1");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 1
            })
        );
        assert_eq!(reader.posistion(), 2);
    }

    #[test]
    fn parse_end_ignore_rest() {
        let mut reader = Reader::new("-10a");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 0,
                length: 10
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_start_greater_than_end_as_error() {
        let mut reader = Reader::new("6-5");
        assert_eq!(
            Range::parse(&mut reader),
            Err(ParseError {
                message: "Range end cannot precede start",
                position: 2,
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_start_equals_to_end() {
        let mut reader = Reader::new("5-5");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 4,
                length: 1
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_start_less_than_end() {
        let mut reader = Reader::new("4-5");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 3,
                length: 2
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_start_less_than_end_ignore_rest() {
        let mut reader = Reader::new("2-10a");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 1,
                length: 9
            })
        );
        assert_eq!(reader.posistion(), 4);
    }

    #[test]
    fn parse_united_start_and_end() {
        let mut reader = Reader::new("100");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 99,
                length: 1
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_united_start_and_end_ignore_rest() {
        let mut reader = Reader::new("100a");
        assert_eq!(
            Range::parse(&mut reader),
            Ok(Range {
                offset: 99,
                length: 1
            })
        );
        assert_eq!(reader.posistion(), 3);
    }
}
