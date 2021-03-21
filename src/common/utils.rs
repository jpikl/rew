use std::io::{Error, ErrorKind, Result};

static mut STATIC_STRINGS: Vec<String> = Vec::new();

pub fn into_static_str(value: String) -> &'static str {
    unsafe {
        // Well, this is ugly but the current usage should be actually safe:
        // 1) It's used only by cli.rs to generate static strings for clap attributes.
        // 2) Values are never modified after being pushed to vector.
        // 3) Vectors is only modified / acessed by a single thread.
        STATIC_STRINGS.push(value);
        STATIC_STRINGS
            .last()
            .expect("Expected at least one static string result")
            .as_str()
    }
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

    #[test]
    fn into_static_str() {
        use super::*;

        assert_eq!(into_static_str("abc".into()), "abc");
        assert_eq!(into_static_str("def".into()), "def");
    }

    mod str_from_utf8 {
        use super::*;
        use crate::testing::unpack_io_error;

        #[test]
        fn ok() {
            assert_eq!(
                str_from_utf8(&[b'a', b'b', b'c'][..]).map_err(unpack_io_error),
                Ok("abc")
            );
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
