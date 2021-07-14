use crate::pattern::char::Char;
use crate::pattern::integer::{parse_integer, ParsableInt};
use crate::pattern::parse::{BaseResult, Error, ErrorKind, Result};
use crate::pattern::reader::Reader;

pub fn parse_index<T: ParsableInt>(reader: &mut Reader<Char>) -> Result<T> {
    let position = reader.position();
    let value = parse_integer(reader)?;

    shift_index(value).map_err(|kind| Error {
        kind,
        range: position..reader.position(),
    })
}

pub fn shift_index<T: ParsableInt>(value: T) -> BaseResult<T> {
    if value >= T::one() {
        Ok(value - T::one())
    } else {
        Err(ErrorKind::IndexZero)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Index;
    use crate::utils::IndexRange;
    use test_case::test_case;

    mod parse_index {
        use test_case::test_case;

        use super::*;

        #[test_case("abc",  0..3, ErrorKind::ExpectedNumber ; "invalid")]
        #[test_case("0abc", 0..1, ErrorKind::IndexZero      ; "zero")]
        fn err(input: &str, range: IndexRange, kind: ErrorKind) {
            assert_eq!(
                parse_index::<Index>(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("1",      0   ; "one")]
        #[test_case("123abc", 122 ; "multiple digits and chars")]
        fn ok(input: &str, result: Index) {
            assert_eq!(parse_index(&mut Reader::from(input)), Ok(result));
        }
    }

    #[test_case(0, Err(ErrorKind::IndexZero) ; "zero")]
    #[test_case(1, Ok(0)                     ; "positive")]
    fn shift_index(index: Index, result: BaseResult<usize>) {
        assert_eq!(super::shift_index(index), result)
    }
}
