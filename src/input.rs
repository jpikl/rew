use std::io::{BufRead, Error, ErrorKind, Result};

pub enum Delimiter {
    Newline,
    Nul,
    None,
}

pub struct Splitter<T: BufRead> {
    reader: T,
    delimiter: Delimiter,
    buffer: Vec<u8>,
}

impl<T: BufRead> Splitter<T> {
    pub fn new(reader: T, delimiter: Delimiter) -> Self {
        Self {
            reader,
            delimiter,
            buffer: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Result<Option<&str>> {
        self.buffer.clear();

        let result = match self.delimiter {
            Delimiter::Newline => self.reader.read_until(b'\n', &mut self.buffer),
            Delimiter::Nul => self.reader.read_until(0, &mut self.buffer),
            Delimiter::None => self.reader.read_to_end(&mut self.buffer),
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
    use std::error::Error;

    #[test]
    fn read__empty() {
        assert_eq!(
            Splitter::new(&[][0..0], Delimiter::Newline).read().unwrap(),
            None
        );
        assert_eq!(
            Splitter::new(&[][0..0], Delimiter::Nul).read().unwrap(),
            None
        );
        assert_eq!(
            Splitter::new(&[][0..0], Delimiter::None).read().unwrap(),
            None
        );
    }

    #[test]
    fn read_newline_delimiter_lf() {
        let mut splitter = Splitter::new("abc\0\n\0def\nghi".as_bytes(), Delimiter::Newline);
        assert_eq!(splitter.read().unwrap(), Some("abc\0"));
        assert_eq!(splitter.read().unwrap(), Some("\0def"));
        assert_eq!(splitter.read().unwrap(), Some("ghi"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_newline_delimiter_lf_end() {
        let mut splitter = Splitter::new("abc\0\n\0def\n".as_bytes(), Delimiter::Newline);
        assert_eq!(splitter.read().unwrap(), Some("abc\0"));
        assert_eq!(splitter.read().unwrap(), Some("\0def"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_newline_delimiter_crlf() {
        let mut splitter = Splitter::new("abc\0\r\n\0def\r\nghi".as_bytes(), Delimiter::Newline);
        assert_eq!(splitter.read().unwrap(), Some("abc\0"));
        assert_eq!(splitter.read().unwrap(), Some("\0def"));
        assert_eq!(splitter.read().unwrap(), Some("ghi"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_newline_delimiter_crlf_end() {
        let mut splitter = Splitter::new("abc\0\r\n\0def\r\n".as_bytes(), Delimiter::Newline);
        assert_eq!(splitter.read().unwrap(), Some("abc\0"));
        assert_eq!(splitter.read().unwrap(), Some("\0def"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_nul_delimiter() {
        let mut splitter = Splitter::new("abc\n\0\ndef\0ghi".as_bytes(), Delimiter::Nul);
        assert_eq!(splitter.read().unwrap(), Some("abc\n"));
        assert_eq!(splitter.read().unwrap(), Some("\ndef"));
        assert_eq!(splitter.read().unwrap(), Some("ghi"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_nul_delimiter_end() {
        let mut splitter = Splitter::new("abc\n\0\ndef\0".as_bytes(), Delimiter::Nul);
        assert_eq!(splitter.read().unwrap(), Some("abc\n"));
        assert_eq!(splitter.read().unwrap(), Some("\ndef"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_none_delimiter() {
        let mut splitter = Splitter::new("abc\n\0def".as_bytes(), Delimiter::None);
        assert_eq!(splitter.read().unwrap(), Some("abc\n\0def"));
        assert_eq!(splitter.read().unwrap(), None);
    }

    #[test]
    fn read_utf8_error() {
        assert_eq!(
            Splitter::new(&[0, 159, 146, 150][..], Delimiter::None)
                .read()
                .map_err(|e| (e.kind(), e.to_string())),
            Err((
                ErrorKind::InvalidData,
                String::from("Input does not have UTF-8 encoding (offset: 1)")
            ))
        );
    }
}
