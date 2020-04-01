use crate::pattern::error::ParseError;
use crate::pattern::reader::Reader;

pub fn parse_usize(reader: &mut Reader) -> Result<usize, ParseError> {
    match reader.peek() {
        Some('0') => {
            reader.read();
            Ok(0)
        }
        Some(ch @ '1'..='9') => {
            let mut number = ch.to_digit(10).unwrap() as usize;
            reader.read();
            while let Some(ch @ '0'..='9') = reader.peek() {
                number = 10 * number + ch.to_digit(10).unwrap() as usize;
                reader.read();
            }
            Ok(number)
        }
        _ => Err(ParseError {
            message: "Expected number",
            position: 0,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_as_error() {
        let mut reader = Reader::new("");
        assert_eq!(
            parse_usize(&mut reader),
            Err(ParseError {
                message: "Expected number",
                position: 0,
            })
        );
        assert_eq!(reader.posistion(), 0);
    }

    #[test]
    fn parse_non_digit_as_error() {
        let mut reader = Reader::new("a");
        assert_eq!(
            parse_usize(&mut reader),
            Err(ParseError {
                message: "Expected number",
                position: 0,
            })
        );
        assert_eq!(reader.posistion(), 0);
    }

    #[test]
    fn parse_zero() {
        let mut reader = Reader::new("0");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_zero_ignore_rest() {
        let mut reader = Reader::new("0a");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_only_a_first_zero() {
        let mut reader = Reader::new("00");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_positive_number_single_digit() {
        let mut reader = Reader::new("1");
        assert_eq!(parse_usize(&mut reader), Ok(1));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_positive_number_single_digit_ignore_rest() {
        let mut reader = Reader::new("1a");
        assert_eq!(parse_usize(&mut reader), Ok(1));
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_positive_number_multiple_digits() {
        let mut reader = Reader::new("1234567890");
        assert_eq!(parse_usize(&mut reader), Ok(1234567890));
        assert_eq!(reader.posistion(), 10);
    }

    #[test]
    fn parse_positive_number_multiple_digits_ignore_rest() {
        let mut reader = Reader::new("1234567890a");
        assert_eq!(parse_usize(&mut reader), Ok(1234567890));
        assert_eq!(reader.posistion(), 10);
    }
}
