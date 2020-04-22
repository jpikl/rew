use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::reader::Reader;

pub fn parse_usize(reader: &mut Reader) -> ParseResult<usize> {
    match reader.peek_value() {
        Some('0') => {
            reader.read_value();
            Ok(0)
        }
        Some(ch @ '1'..='9') => {
            let mut number = ch.to_digit(10).unwrap() as usize;
            reader.read_value();
            while let Some(ch @ '0'..='9') = reader.peek_value() {
                number = 10 * number + ch.to_digit(10).unwrap() as usize;
                reader.read_value();
            }
            Ok(number)
        }
        _ => Err(ParseError {
            kind: ParseErrorKind::ExpectedNumber,
            start: reader.position(),
            end: reader.end(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_error() {
        let mut reader = Reader::from("");
        assert_eq!(
            parse_usize(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedNumber,
                start: 0,
                end: 0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn non_digit_error() {
        let mut reader = Reader::from("ab");
        assert_eq!(
            parse_usize(&mut reader),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedNumber,
                start: 0,
                end: 2,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn zero() {
        let mut reader = Reader::from("0");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn zero_ignore_rest() {
        let mut reader = Reader::from("0a");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn only_a_first_zero() {
        let mut reader = Reader::from("00");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn positive_number_single_digit() {
        let mut reader = Reader::from("1");
        assert_eq!(parse_usize(&mut reader), Ok(1));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn positive_number_single_digit_ignore_rest() {
        let mut reader = Reader::from("1a");
        assert_eq!(parse_usize(&mut reader), Ok(1));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn positive_number_multiple_digits() {
        let mut reader = Reader::from("1234567890");
        assert_eq!(parse_usize(&mut reader), Ok(1_234_567_890));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn positive_number_multiple_digits_ignore_rest() {
        let mut reader = Reader::from("1234567890a");
        assert_eq!(parse_usize(&mut reader), Ok(1_234_567_890));
        assert_eq!(reader.position(), 10);
    }
}
