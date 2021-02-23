use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use num_traits::PrimInt;
use std::convert::TryFrom;
use std::fmt::Display;

pub const fn get_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub trait ParsableInt: TryFrom<u32> + PrimInt + Display {}

impl<T: TryFrom<u32> + PrimInt + Display> ParsableInt for T {}

pub fn parse_integer<T: ParsableInt>(reader: &mut Reader<Char>) -> Result<T> {
    if let Some(ch @ '0'..='9') = reader.peek_char() {
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
                        kind: ErrorKind::IntegerOverflow(T::max_value().to_string()),
                        range: position..reader.position(),
                    })
                }
            }

            match number.checked_add(&parse_digit(ch)) {
                Some(result) => number = result,
                None => {
                    return Err(Error {
                        kind: ErrorKind::IntegerOverflow(T::max_value().to_string()),
                        range: position..reader.position(),
                    })
                }
            }
        }

        Ok(number)
    } else {
        Err(Error {
            kind: ErrorKind::ExpectedNumber,
            range: reader.position()..reader.end(),
        })
    }
}

fn parse_digit<T: TryFrom<u32>>(value: char) -> T {
    // This should never fail even for T = u8, the caller makes sure value is a digit
    parse_u32(value.to_digit(10).expect("Expected a digit"))
}

fn parse_u32<T: TryFrom<u32>>(value: u32) -> T {
    // This should never fail even for T = u8, the caller makes sure value is a digit
    // We are not using .expect() because it requires TryFrom::Error to implement Display
    // which would introduce another unnecessary function template parameter.
    match T::try_from(value) {
        Ok(result) => result,
        _ => panic!("Expected to convert from u32"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_bits() {
        assert_eq!(get_bits::<u8>(), 8);
        assert_eq!(get_bits::<u16>(), 16);
        assert_eq!(get_bits::<u32>(), 32);
        assert_eq!(get_bits::<u64>(), 64);
        assert_eq!(get_bits::<u128>(), 128);
    }

    mod parse_integer {
        use super::*;

        #[test]
        fn empty() {
            let mut reader = Reader::from("");
            assert_eq!(
                parse_integer::<usize>(&mut reader),
                Err(Error {
                    kind: ErrorKind::ExpectedNumber,
                    range: 0..0,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn alpha() {
            let mut reader = Reader::from("ab");
            assert_eq!(
                parse_integer::<usize>(&mut reader),
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
            assert_eq!(parse_integer(&mut reader), Ok(0));
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn zero_then_zero() {
            let mut reader = Reader::from("00");
            assert_eq!(parse_integer(&mut reader), Ok(0));
            assert_eq!(reader.position(), 2);
        }

        #[test]
        fn zero_then_nonzero() {
            let mut reader = Reader::from("01");
            assert_eq!(parse_integer(&mut reader), Ok(1));
            assert_eq!(reader.position(), 2);
        }

        #[test]
        fn zero_then_alpha() {
            let mut reader = Reader::from("0a");
            assert_eq!(parse_integer(&mut reader), Ok(0));
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn single_digit() {
            let mut reader = Reader::from("1");
            assert_eq!(parse_integer(&mut reader), Ok(1));
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn single_digit_then_alpha() {
            let mut reader = Reader::from("1a");
            assert_eq!(parse_integer(&mut reader), Ok(1));
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn multiple_digits() {
            let mut reader = Reader::from("1234567890");
            assert_eq!(parse_integer(&mut reader), Ok(1_234_567_890));
            assert_eq!(reader.position(), 10);
        }

        #[test]
        fn multiple_digits_then_alpha() {
            let mut reader = Reader::from("1234567890a");
            assert_eq!(parse_integer(&mut reader), Ok(1_234_567_890));
            assert_eq!(reader.position(), 10);
        }

        #[test]
        fn mul_overflow() {
            let mut reader = Reader::from("25500");
            assert_eq!(
                parse_integer::<u8>(&mut reader),
                Err(Error {
                    kind: ErrorKind::IntegerOverflow(String::from("255")),
                    range: 0..4
                })
            );
            assert_eq!(reader.position(), 4);
        }

        #[test]
        fn add_overflow() {
            let mut reader = Reader::from("2560");
            assert_eq!(
                parse_integer::<u8>(&mut reader),
                Err(Error {
                    kind: ErrorKind::IntegerOverflow(String::from("255")),
                    range: 0..3
                })
            );
            assert_eq!(reader.position(), 3);
        }
    }
}
