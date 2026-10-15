use std::io::{Error, ErrorKind, Result};

pub fn into_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

pub fn str_from_utf8(data: &[u8]) -> Result<&str> {
    match std::str::from_utf8(data) {
        Ok(str) => Ok(str),
        Err(error) => Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Value does not have UTF-8 encoding (offset {})",
                error.valid_up_to()
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] // Do not use test_case, this is expected to work only for a single thread.
    fn into_static_str() {
        let str_1 = super::into_static_str("a".into());
        let str_2 = super::into_static_str("ab".into());
        let str_3 = super::into_static_str("abc".into());

        assert_eq!(str_1, "a");
        assert_eq!(str_2, "ab");
        assert_eq!(str_3, "abc");
    }

    mod str_from_utf8 {
        use super::*;
        use crate::testing::unpack_io_error;

        #[test]
        fn ok() {
            assert_eq!(str_from_utf8(b"abc").map_err(unpack_io_error), Ok("abc"));
        }

        #[test]
        fn err() {
            assert_eq!(
                str_from_utf8(&[0, 159, 146, 150][..]).map_err(unpack_io_error),
                Err((
                    ErrorKind::InvalidData,
                    "Value does not have UTF-8 encoding (offset 1)".into()
                ))
            );
        }
    }
}
