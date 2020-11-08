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

        let result = match self.delimiter {
            Delimiter::Newline => self.input.read_until(b'\n', &mut self.buffer),
            Delimiter::Byte(delimiter) => self.input.read_until(delimiter, &mut self.buffer),
            Delimiter::None => self.input.read_to_end(&mut self.buffer),
        };

        match result {
            Ok(0) => Ok(None),
            Ok(mut size) => {
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
            }
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unpack_io_error;

    #[test]
    fn splitter_newline_delimiter_empty() {
        let mut splitter = Splitter::new(&[][..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_newline_delimiter_lf_between() {
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
    fn splitter_newline_delimiter_lf_end() {
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
    fn splitter_newline_delimiter_lf_consecutive() {
        let mut splitter = Splitter::new(&b"\n\n"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_newline_delimiter_crlf_between() {
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
    fn splitter_newline_delimiter_crlf_end() {
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
    fn splitter_newline_delimiter_crlf_consecutive() {
        let mut splitter = Splitter::new(&b"\r\n\n\r\n"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_byte_delimiter_empty() {
        let mut splitter = Splitter::new(&[][..], Delimiter::Byte(0));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_byte_delimiter_between() {
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
    fn splitter_byte_delimiter_end() {
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
    fn splitter_byte_delimiter_consecutive() {
        let mut splitter = Splitter::new(&b"\0\0"[..], Delimiter::Byte(0));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_none_delimiter_empty() {
        let mut splitter = Splitter::new(&[][..], Delimiter::None);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_none_delimiter() {
        let mut splitter = Splitter::new(&b"abc\n\0def"[..], Delimiter::None);
        assert_eq!(
            splitter.read().map_err(unpack_io_error),
            Ok(Some(("abc\n\0def", 8)))
        );
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_utf8_error() {
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
