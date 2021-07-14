use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use num_traits::PrimInt;
use std::fmt::{Debug, Display};
use std::str::FromStr;

pub trait ParsableInt: PrimInt + FromStr + Display + Debug {}

impl<T: PrimInt + FromStr + Display + Debug> ParsableInt for T {}

pub fn parse_integer<T: ParsableInt>(reader: &mut Reader<Char>) -> Result<T> {
    let position = reader.position();
    let mut buffer = String::new();

    while let Some(digit @ '0'..='9') = reader.peek_char() {
        buffer.push(digit);
        reader.seek();
    }

    if buffer.is_empty() {
        Err(Error {
            kind: ErrorKind::ExpectedNumber,
            range: reader.position()..reader.end(),
        })
    } else {
        buffer.parse::<T>().map_err(|_| Error {
            // This is the only possible reason for an error
            kind: ErrorKind::IntegerOverflow(T::max_value().to_string()),
            range: position..reader.position(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_integer {
        use super::*;
        use crate::utils::IndexRange;
        use test_case::test_case;

        #[test_case("",     0..0, ErrorKind::ExpectedNumber                ; "empty")]
        #[test_case("ab",   0..2, ErrorKind::ExpectedNumber                ; "alpha")]
        #[test_case("256",  0..3, ErrorKind::IntegerOverflow("255".into()) ; "overflow")]
        #[test_case("256a", 0..3, ErrorKind::IntegerOverflow("255".into()) ; "overflow then alpha")]
        fn err(input: &str, range: IndexRange, kind: ErrorKind) {
            assert_eq!(
                parse_integer::<u8>(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("0",    0,   1 ; "zero")]
        #[test_case("00",   0,   2 ; "zero then zero")]
        #[test_case("01",   1,   2 ; "zero then nonzero")]
        #[test_case("0a",   0,   1 ; "zero then alpha")]
        #[test_case("1",    1,   1 ; "single digit")]
        #[test_case("1a",   1,   1 ; "single digit then alpha")]
        #[test_case("123",  123, 3 ; "multiple digits")]
        #[test_case("123a", 123, 3 ; "multiple digits then alpha")]
        fn ok(input: &str, output: usize, position: usize) {
            let mut reader = Reader::from(input);
            assert_eq!(parse_integer(&mut reader), Ok(output));
            assert_eq!(reader.position(), position);
        }
    }
}
