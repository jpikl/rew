use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;

pub fn parse_usize(reader: &mut Reader<Char>) -> Result<usize> {
    match reader.peek_char() {
        Some('0') => {
            reader.seek();
            Ok(0)
        }
        Some(ch @ '1'..='9') => {
            let mut number = ch.to_digit(10).unwrap() as usize;
            reader.seek();
            while let Some(ch @ '0'..='9') = reader.peek_char() {
                number = 10 * number + ch.to_digit(10).unwrap() as usize;
                reader.seek();
            }
            Ok(number)
        }
        _ => Err(Error {
            kind: ErrorKind::ExpectedNumber,
            range: reader.position()..reader.end(),
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
            Err(Error {
                kind: ErrorKind::ExpectedNumber,
                range: 0..0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn no_digits_error() {
        let mut reader = Reader::from("ab");
        assert_eq!(
            parse_usize(&mut reader),
            Err(Error {
                kind: ErrorKind::ExpectedNumber,
                range: 0..2,
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
    fn zero_ignore_following_zeros() {
        let mut reader = Reader::from("00");
        assert_eq!(parse_usize(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn single_digit() {
        let mut reader = Reader::from("1");
        assert_eq!(parse_usize(&mut reader), Ok(1));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn single_digit_ignore_rest() {
        let mut reader = Reader::from("1a");
        assert_eq!(parse_usize(&mut reader), Ok(1));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn multiple_digits() {
        let mut reader = Reader::from("1234567890");
        assert_eq!(parse_usize(&mut reader), Ok(1_234_567_890));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn multiple_digits_ignore_rest() {
        let mut reader = Reader::from("1234567890a");
        assert_eq!(parse_usize(&mut reader), Ok(1_234_567_890));
        assert_eq!(reader.position(), 10);
    }
}
