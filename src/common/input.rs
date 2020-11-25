use std::io::{BufRead, Error, ErrorKind, Result};

pub enum Delimiter {
    Newline,
    Byte(u8),
    None,
}

pub struct Splitter<I: BufRead> {
    input: I,
    delimiter: Delimiter,
    buffer: Vec<u8>,
}

impl<I: BufRead> Splitter<I> {
    pub fn new(input: I, delimiter: Delimiter) -> Self {
        Self {
            input,
            delimiter,
            buffer: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Result<Option<(&str, usize)>> {
        self.buffer.clear();

        let mut size = match self.delimiter {
            Delimiter::Newline => self.input.read_until(b'\n', &mut self.buffer)?,
            Delimiter::Byte(delimiter) => self.input.read_until(delimiter, &mut self.buffer)?,
            Delimiter::None => self.input.read_to_end(&mut self.buffer)?,
        };

        if size > 0 {
            let orig_size = size;

            match self.delimiter {
                Delimiter::Newline => {
                    if size > 0 && self.buffer[size - 1] == b'\n' {
                        size -= 1;
                        if size > 0 && self.buffer[size - 1] == b'\r' {
                            size -= 1;
                        }
                    }
                }
                Delimiter::Byte(delimiter) => {
                    if size > 0 && self.buffer[size - 1] == delimiter {
                        size -= 1;
                    }
                }
                Delimiter::None => {}
            }

            match std::str::from_utf8(&self.buffer[..size]) {
                Ok(str) => Ok(Some((str, orig_size))),
                Err(error) => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Input does not have UTF-8 encoding (offset {})",
                        error.valid_up_to()
                    ),
                )),
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unpack_io_error;

    mod newline_delimiter {
        use super::*;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], Delimiter::Newline);
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        mod lf_only {
            use super::*;

            #[test]
            fn between() {
                let mut splitter = Splitter::new(&b"abc\0\n\0def"[..], Delimiter::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 5)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 4)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between_and_end() {
                let mut splitter = Splitter::new(&b"abc\0\n\0def\n"[..], Delimiter::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 5)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 5)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn consecutive() {
                let mut splitter = Splitter::new(&b"\n\n"[..], Delimiter::Newline);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }
        }

        mod cr_and_lf {
            use super::*;

            #[test]
            fn between() {
                let mut splitter = Splitter::new(&b"abc\0\r\n\0def"[..], Delimiter::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 6)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 4)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between_and_end() {
                let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\n"[..], Delimiter::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 6)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 6)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn consecutive() {
                let mut splitter = Splitter::new(&b"\r\n\n\r\n"[..], Delimiter::Newline);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }
        }
    }

    mod byte_delimiter {
        use super::*;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], Delimiter::Byte(0));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn between() {
            let mut splitter = Splitter::new(&b"abc\n\0\ndef"[..], Delimiter::Byte(0));
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n", 5)))
            );
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("\ndef", 4)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn between_and_end() {
            let mut splitter = Splitter::new(&b"abc\n\0\ndef\0"[..], Delimiter::Byte(0));
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n", 5)))
            );
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("\ndef", 5)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn consecutive() {
            let mut splitter = Splitter::new(&b"\0\0"[..], Delimiter::Byte(0));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }
    }

    mod none_delimiter {
        use super::*;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], Delimiter::None);
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn nonempty() {
            let mut splitter = Splitter::new(&b"abc\n\0def"[..], Delimiter::None);
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n\0def", 8)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }
    }

    #[test]
    fn non_utf8() {
        assert_eq!(
            Splitter::new(&[0, 159, 146, 150][..], Delimiter::None)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::InvalidData,
                String::from("Input does not have UTF-8 encoding (offset 1)")
            ))
        );
    }
}
