use crate::input::{Delimiter, Splitter};
use crate::symbols::{DIFF_IN, DIFF_OUT};
use std::io::{BufRead, Error, ErrorKind, Result};
use std::path::PathBuf;

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
