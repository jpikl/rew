use crate::pattern::parse::number::parse_usize;
use crate::pattern::parse::reader::Reader;
use crate::pattern::parse::types::ParseError;
use crate::pattern::types::Range;

impl Range {
    pub fn parse(reader: &mut Reader) -> Result<Self, ParseError> {
        match reader.peek() {
            Some('0'..='9') => {
                let from = parse_usize(reader)?;
                if let Some('-') = reader.peek() {
                    reader.read();
                    if let Some('0'..='9') = reader.peek() {
                        let to = parse_usize(reader)?;
                        Ok(Range::FromTo(from, to))
                    } else {
                        Ok(Range::From(from))
                    }
                } else {
                    Ok(Range::FromTo(from, from))
                }
            }
            Some('-') => {
                reader.read();
                if let Some('0'..='9') = reader.peek() {
                    let to = parse_usize(reader)?;
                    Ok(Range::To(to))
                } else {
                    Ok(Range::Full)
                }
            }
            _ => Err(ParseError {
                message: "Expected range",
                position: reader.posistion(),
            }),
        }
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
        assert_eq!(Range::parse(&mut reader), Ok(Range::Full));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_zero_ignore_rest() {
        let mut reader = Reader::new("-a");
        assert_eq!(Range::parse(&mut reader), Ok(Range::Full));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_from() {
        let mut reader = Reader::new("12-");
        assert_eq!(Range::parse(&mut reader), Ok(Range::From(12)));
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_from_ignore_rest() {
        let mut reader = Reader::new("12-a");
        assert_eq!(Range::parse(&mut reader), Ok(Range::From(12)));
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_to() {
        let mut reader = Reader::new("-34");
        assert_eq!(Range::parse(&mut reader), Ok(Range::To(34)));
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_to_ignore_rest() {
        let mut reader = Reader::new("-34a");
        assert_eq!(Range::parse(&mut reader), Ok(Range::To(34)));
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_from_to() {
        let mut reader = Reader::new("12-34");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(12, 34)));
        assert_eq!(reader.posistion(), 5);
    }

    #[test]
    fn parse_from_to_ignore_rest() {
        let mut reader = Reader::new("12-34a");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(12, 34)));
        assert_eq!(reader.posistion(), 5);
    }

    #[test]
    fn parse_from_to_same() {
        let mut reader = Reader::new("7890");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(7890, 7890)));
        assert_eq!(reader.posistion(), 4);
    }

    #[test]
    fn parse_from_to_same_ignore_rest() {
        let mut reader = Reader::new("7890a");
        assert_eq!(Range::parse(&mut reader), Ok(Range::FromTo(7890, 7890)));
        assert_eq!(reader.posistion(), 4);
    }
}
