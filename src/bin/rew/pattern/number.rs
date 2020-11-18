use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use num_traits::PrimInt;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};

pub fn parse_number<T, E>(reader: &mut Reader<Char>) -> Result<T>
where
    T: TryFrom<u32, Error = E> + PrimInt + Display,
    E: Debug,
{
    match reader.peek_char() {
        Some('0') => {
            reader.seek();
            Ok(T::zero())
        }
        Some(ch @ '1'..='9') => {
            let position = reader.position();
            reader.seek();

            let base: T = parse_u32(10);
            let mut number: T = parse_digit(ch);

            while let Some(ch @ '0'..='9') = reader.peek_char() {
                reader.seek();

                match number.checked_mul(&base) {
                    Some(result) => number = result,
                    None => {
                        return Err(Error {
                            kind: ErrorKind::NumberOverflow(T::max_value().to_string()),
                            range: position..reader.position(),
                        })
                    }
                }

                match number.checked_add(&parse_digit(ch)) {
                    Some(result) => number = result,
                    None => {
                        return Err(Error {
                            kind: ErrorKind::NumberOverflow(T::max_value().to_string()),
                            range: position..reader.position(),
                        })
                    }
                }
            }

            Ok(number)
        }
        _ => Err(Error {
            kind: ErrorKind::ExpectedNumber,
            range: reader.position()..reader.end(),
        }),
    }
}

fn parse_digit<T: TryFrom<u32, Error = E>, E: Debug>(value: char) -> T {
    // This should never fail even for T = u8, the caller makes sure value is a digit
    parse_u32(value.to_digit(10).expect("Expected a digit"))
}

fn parse_u32<T: TryFrom<u32, Error = E>, E: Debug>(value: u32) -> T {
    // This should never fail even for T = u8, the caller makes sure value is a digit
    T::try_from(value).expect("Expected to convert from u32")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_error() {
        let mut reader = Reader::from("");
        assert_eq!(
            parse_number::<usize, _>(&mut reader),
            Err(Error {
                kind: ErrorKind::ExpectedNumber,
                range: 0..0,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_no_digits_error() {
        let mut reader = Reader::from("ab");
        assert_eq!(
            parse_number::<usize, _>(&mut reader),
            Err(Error {
                kind: ErrorKind::ExpectedNumber,
                range: 0..2,
            })
        );
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn parse_zero() {
        let mut reader = Reader::from("0");
        assert_eq!(parse_number(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_zero_ignore_rest() {
        let mut reader = Reader::from("0a");
        assert_eq!(parse_number(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_zero_ignore_following_zeros() {
        let mut reader = Reader::from("00");
        assert_eq!(parse_number(&mut reader), Ok(0));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_single_digit() {
        let mut reader = Reader::from("1");
        assert_eq!(parse_number(&mut reader), Ok(1));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_single_digit_ignore_rest() {
        let mut reader = Reader::from("1a");
        assert_eq!(parse_number(&mut reader), Ok(1));
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_multiple_digits() {
        let mut reader = Reader::from("1234567890");
        assert_eq!(parse_number(&mut reader), Ok(1_234_567_890));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn parse_multiple_digits_ignore_rest() {
        let mut reader = Reader::from("1234567890a");
        assert_eq!(parse_number(&mut reader), Ok(1_234_567_890));
        assert_eq!(reader.position(), 10);
    }

    #[test]
    fn parse_mul_overflow_error() {
        let mut reader = Reader::from("25500");
        assert_eq!(
            parse_number::<u8, _>(&mut reader),
            Err(Error {
                kind: ErrorKind::NumberOverflow(String::from("255")),
                range: 0..4
            })
        );
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn parse_add_overflow_error() {
        let mut reader = Reader::from("2560");
        assert_eq!(
            parse_number::<u8, _>(&mut reader),
            Err(Error {
                kind: ErrorKind::NumberOverflow(String::from("255")),
                range: 0..3
            })
        );
        assert_eq!(reader.position(), 3);
    }
}
