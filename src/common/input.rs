use crate::symbols::{DIFF_IN, DIFF_OUT};
use std::io::{BufRead, Error, ErrorKind, Result};
use std::path::PathBuf;

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

pub struct PathDiff<I: BufRead> {
    splitter: Splitter<I>,
}

impl<I: BufRead> PathDiff<I> {
    pub fn new(input: I, delimiter: Delimiter) -> Self {
        Self {
            splitter: Splitter::new(input, delimiter),
        }
    }

    pub fn read(&mut self) -> Result<Option<(PathBuf, PathBuf)>> {
        let in_path = match self.splitter.read()? {
            Some(value) => extract_prefixed_path(value, DIFF_IN)?,
            None => return Ok(None),
        };
        let out_path = extract_prefixed_path(self.splitter.read()?.unwrap_or(""), DIFF_OUT)?;
        Ok(Some((in_path, out_path)))
    }
}

fn extract_prefixed_path(value: &str, prefix: char) -> Result<PathBuf> {
    if let Some(first_char) = value.chars().next() {
        if first_char == prefix {
            let path = &value[prefix.len_utf8()..];
            if path.is_empty() {
                Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!(
                        "Expected '{}' followed by a path but got only '{}'",
                        prefix, value
                    ),
                ))
            } else {
                Ok(PathBuf::from(path))
            }
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Expected '{}' followed by a path but got '{}'",
                    prefix, value
                ),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!("Expected '{}' followed by a path", prefix),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unpack_io_error;
    use indoc::indoc;

    #[test]
    fn splitter_empty() {
        assert_eq!(
            Splitter::new(&[][..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Ok(None)
        );
        assert_eq!(
            Splitter::new(&[][..], Delimiter::Nul)
                .read()
                .map_err(unpack_io_error),
            Ok(None)
        );
        assert_eq!(
            Splitter::new(&[][..], Delimiter::None)
                .read()
                .map_err(unpack_io_error),
            Ok(None)
        );
    }

    #[test]
    fn splitter_newline_delimiter_lf() {
        let mut splitter = Splitter::new(&b"abc\0\n\0def\nghi"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("ghi")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_newline_delimiter_lf_end() {
        let mut splitter = Splitter::new(&b"abc\0\n\0def\n"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_newline_delimiter_crlf() {
        let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\nghi"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("ghi")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_newline_delimiter_crlf_end() {
        let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\n"[..], Delimiter::Newline);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("abc\0")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("\0def")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_nul_delimiter() {
        let mut splitter = Splitter::new(&b"abc\n\0\ndef\0ghi"[..], Delimiter::Nul);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("abc\n")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("\ndef")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("ghi")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_nul_delimiter_end() {
        let mut splitter = Splitter::new(&b"abc\n\0\ndef\0"[..], Delimiter::Nul);
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("abc\n")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some("\ndef")));
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn splitter_none_delimiter() {
        let mut splitter = Splitter::new(&b"abc\n\0def"[..], Delimiter::None);
        assert_eq!(
            splitter.read().map_err(unpack_io_error),
            Ok(Some("abc\n\0def"))
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
                String::from("Input does not have UTF-8 encoding (offset: 1)")
            ))
        );
    }

    #[test]
    fn path_diff_empty() {
        assert_eq!(
            PathDiff::new(&[][..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Ok(None)
        );
    }

    #[test]
    fn path_diff_read() {
        let input = indoc! {"
            <abc
            >def
            < g h i 
            > j k l 
        "};
        let mut path_diff = PathDiff::new(input.as_bytes(), Delimiter::Newline);
        assert_eq!(
            path_diff.read().map_err(unpack_io_error),
            Ok(Some((PathBuf::from("abc"), PathBuf::from("def"))))
        );
        assert_eq!(
            path_diff.read().map_err(unpack_io_error),
            Ok(Some((PathBuf::from(" g h i "), PathBuf::from(" j k l "))))
        );
        assert_eq!(path_diff.read().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn path_diff_empty_in() {
        assert_eq!(
            PathDiff::new(&b"<"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected '<' followed by a path but got only '<'")
            ))
        )
    }

    #[test]
    fn path_diff_invalid_in() {
        assert_eq!(
            PathDiff::new(&b"abc"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::InvalidData,
                String::from("Expected '<' followed by a path but got 'abc'")
            ))
        )
    }

    #[test]
    fn path_diff_no_out() {
        assert_eq!(
            PathDiff::new(&b"<abc"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected '>' followed by a path")
            ))
        )
    }

    #[test]
    fn path_diff_empty_out() {
        assert_eq!(
            PathDiff::new(&b"<abc\n>"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected '>' followed by a path but got only '>'")
            ))
        )
    }

    #[test]
    fn path_diff_invalid_out() {
        assert_eq!(
            PathDiff::new(&b"<abc\ndef"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::InvalidData,
                String::from("Expected '>' followed by a path but got 'def'")
            ))
        )
    }
}
