use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use num_traits::PrimInt;
use std::convert::TryFrom;
use std::fmt::{Debug, Display};

pub const fn get_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

pub trait ParsableInt: TryFrom<u32> + PrimInt + Display + Debug {}

impl<T: TryFrom<u32> + PrimInt + Display + Debug> ParsableInt for T {}

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
    use test_case::test_case;

    #[test_case(0u8, 8; "8-bit")]
    #[test_case(0u16, 16; "16-bit")]
    #[test_case(0u32, 32; "32-bit")]
    #[test_case(0u64, 64; "64-bit")]
    #[test_case(0u128, 128; "128-bit")]
    fn gets_bits<T>(_: T, bits: usize) {
        assert_eq!(super::get_bits::<T>(), bits);
    }

    mod parse_integer {
        use super::*;
        use crate::utils::ByteRange;
        use test_case::test_case;

        #[test_case("", ErrorKind::ExpectedNumber, 0..0; "empty")]
        #[test_case("ab", ErrorKind::ExpectedNumber, 0..2; "alpha")]
        #[test_case("25500", ErrorKind::IntegerOverflow(String::from("255")), 0..4; "mul overflow")]
        #[test_case("2560", ErrorKind::IntegerOverflow(String::from("255")), 0..3; "add overflow")]
        fn err(input: &str, kind: ErrorKind, range: ByteRange) {
            assert_eq!(
                parse_integer::<u8>(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("0", 0, 1; "zero")]
        #[test_case("00", 0, 2; "zero then zero")]
        #[test_case("01", 1, 2; "zero then nonzero")]
        #[test_case("0a", 0, 1; "zero then alpha")]
        #[test_case("1", 1, 1; "single digit")]
        #[test_case("1a", 1, 1; "single digit then alpha")]
        #[test_case("1234567890", 1_234_567_890, 10; "multiple digits")]
        #[test_case("1234567890a", 1_234_567_890, 10; "multiple digits then alpha")]
        fn ok(input: &str, output: usize, position: usize) {
            let mut reader = Reader::from(input);
            assert_eq!(parse_integer(&mut reader), Ok(output));
            assert_eq!(reader.position(), position);
        }
    }
}
