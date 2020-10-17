use std::io::{BufRead, Error, ErrorKind, Result};

pub enum Delimiter {
    Newline,
    Nul,
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

    pub fn read(&mut self) -> Result<Option<&str>> {
        self.buffer.clear();

        let result = match self.delimiter {
            Delimiter::Newline => self.input.read_until(b'\n', &mut self.buffer),
            Delimiter::Nul => self.input.read_until(0, &mut self.buffer),
            Delimiter::None => self.input.read_to_end(&mut self.buffer),
        };

        match result {
            Ok(0) => Ok(None),
            Ok(mut size) => {
                match self.delimiter {
                    Delimiter::Newline => {
                        if self.buffer[size - 1] == b'\n' {
                            size -= 1;
                            if self.buffer[size - 1] == b'\r' {
                                size -= 1;
                            }
                        }
                    }
                    Delimiter::Nul => {
                        if self.buffer[size - 1] == 0 {
                            size -= 1;
                        }
                    }
                    Delimiter::None => {}
                }
                match std::str::from_utf8(&self.buffer[..size]) {
                    Ok(str) => Ok(Some(str)),
                    Err(error) => Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Input does not have UTF-8 encoding (offset: {})",
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

    #[test]
    fn read_empty() {
        assert_eq!(
            Splitter::new(&[][0..0], Delimiter::Newline)
                .read()
                .map_err(map_err),
            Ok(None)
        );
        assert_eq!(
            Splitter::new(&[][0..0], Delimiter::Nul)
                .read()
                .map_err(map_err),
            Ok(None)
        );
        assert_eq!(
            Splitter::new(&[][0..0], Delimiter::None)
                .read()
                .map_err(map_err),
            Ok(None)
        );
    }

    #[test]
    fn read_newline_delimiter_lf() {
        let mut splitter = Splitter::new(&b"abc\0\n\0def\nghi"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("ghi")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_newline_delimiter_lf_end() {
        let mut splitter = Splitter::new(&b"abc\0\n\0def\n"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_newline_delimiter_crlf() {
        let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\nghi"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("ghi")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_newline_delimiter_crlf_end() {
        let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\n"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_nul_delimiter() {
        let mut splitter = Splitter::new(&b"abc\n\0\ndef\0ghi"[..], Delimiter::Nul);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\n")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("\ndef")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("ghi")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_nul_delimiter_end() {
        let mut splitter = Splitter::new(&b"abc\n\0\ndef\0"[..], Delimiter::Nul);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\n")));
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("\ndef")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_none_delimiter() {
        let mut splitter = Splitter::new(&b"abc\n\0def"[..], Delimiter::None);
        assert_eq!(splitter.read().map_err(map_err), Ok(Some("abc\n\0def")));
        assert_eq!(splitter.read().map_err(map_err), Ok(None));
    }

    #[test]
    fn read_utf8_error() {
        assert_eq!(
            Splitter::new(&[0, 159, 146, 150][..], Delimiter::None)
                .read()
                .map_err(map_err),
            Err((
                ErrorKind::InvalidData,
                String::from("Input does not have UTF-8 encoding (offset: 1)")
            ))
        );
    }

    fn map_err(error: Error) -> (ErrorKind, String) {
        (error.kind(), error.to_string())
    }
}
