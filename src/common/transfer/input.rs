use crate::input::{Delimiter, Splitter};
use crate::symbols::{DIFF_IN, DIFF_OUT};
use std::fmt;
use std::io::{BufRead, Error, ErrorKind, Result};
use std::path::PathBuf;

struct Position {
    item: usize,
    offset: usize,
}

impl Position {
    pub fn new() -> Self {
        Self { item: 1, offset: 0 }
    }

    pub fn increment(&mut self, size: usize) {
        self.item += 1;
        self.offset += size;
    }
}

impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "item #{} at offset {}", self.item, self.offset)
    }
}

pub struct PathDiff<I: BufRead> {
    splitter: Splitter<I>,
    position: Position,
}

impl<I: BufRead> PathDiff<I> {
    pub fn new(input: I, delimiter: Delimiter) -> Self {
        Self {
            splitter: Splitter::new(input, delimiter),
            position: Position::new(),
        }
    }

    pub fn read(&mut self) -> Result<Option<(PathBuf, PathBuf)>> {
        let (in_path, in_size) = match self.splitter.read()? {
            Some((value, size)) => (extract_path(value, &self.position, DIFF_IN)?, size),
            None => return Ok(None),
        };
        self.position.increment(in_size);

        let (out_path, out_size) = match self.splitter.read()? {
            Some((value, size)) => (extract_path(value, &self.position, DIFF_OUT)?, size),
            None => return Err(make_unexpected_eof_error(&self.position, DIFF_OUT)),
        };
        self.position.increment(out_size);

        Ok(Some((in_path, out_path)))
    }
}

fn extract_path(value: &str, position: &Position, prefix: char) -> Result<PathBuf> {
    if let Some(first_char) = value.chars().next() {
        if first_char == prefix {
            let path = &value[prefix.len_utf8()..];
            if path.is_empty() {
                Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Expected a path after '{}' ({})", prefix, position),
                ))
            } else {
                Ok(PathBuf::from(path))
            }
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Expected '{}' but got '{}' ({})",
                    prefix, first_char, position
                ),
            ))
        }
    } else {
        Err(make_unexpected_eof_error(position, prefix))
    }
}

fn make_unexpected_eof_error(position: &Position, prefix: char) -> Error {
    Error::new(
        ErrorKind::UnexpectedEof,
        format!("Expected '{}' ({})", prefix, position),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unpack_io_error;
    use indoc::indoc;

    #[test]
    fn position() {
        let mut position = Position::new();
        assert_eq!(position.to_string(), "item #1 at offset 0");
        position.increment(1);
        assert_eq!(position.to_string(), "item #2 at offset 1");
        position.increment(2);
        assert_eq!(position.to_string(), "item #3 at offset 3");
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
    fn path_diff_valid() {
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
    fn path_diff_invalid_in_prefix() {
        assert_eq!(
            PathDiff::new(&b"abc"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::InvalidData,
                String::from("Expected '<' but got 'a' (item #1 at offset 0)")
            ))
        )
    }

    #[test]
    fn path_diff_invalid_out_prefix() {
        assert_eq!(
            PathDiff::new(&b"<abc\ndef"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::InvalidData,
                String::from("Expected '>' but got 'd' (item #2 at offset 5)")
            ))
        )
    }

    #[test]
    fn path_diff_missing_in_path() {
        assert_eq!(
            PathDiff::new(&b"<"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected a path after '<' (item #1 at offset 0)")
            ))
        )
    }

    #[test]
    fn path_diff_missing_out_path() {
        assert_eq!(
            PathDiff::new(&b"<abc\n>"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected a path after '>' (item #2 at offset 5)")
            ))
        )
    }

    #[test]
    fn path_diff_missing_out() {
        assert_eq!(
            PathDiff::new(&b"<abc"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected '>' (item #2 at offset 4)")
            ))
        )
    }

    #[test]
    fn path_diff_empty_out() {
        assert_eq!(
            PathDiff::new(&b"<abc\n\n"[..], Delimiter::Newline)
                .read()
                .map_err(unpack_io_error),
            Err((
                ErrorKind::UnexpectedEof,
                String::from("Expected '>' (item #2 at offset 5)")
            ))
        )
    }
}
