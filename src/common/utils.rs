use std::io::{Error, ErrorKind, Result};

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

    mod str_from_utf8 {
        use super::*;
        use crate::testing::unpack_io_error;

        #[test]
        fn valid() {
            assert_eq!(
                str_from_utf8(&[b'a', b'b', b'c'][..]).map_err(unpack_io_error),
                Ok("abc")
            );
        }

        #[test]
        fn invalid() {
            assert_eq!(
                str_from_utf8(&[0, 159, 146, 150][..]).map_err(unpack_io_error),
                Err((
                    ErrorKind::InvalidData,
                    String::from("Value does not have UTF-8 encoding (offset 1)")
                ))
            );
        }
    }
}
