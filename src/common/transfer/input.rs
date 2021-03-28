use crate::input::{Splitter, Terminator};
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
    pub fn new(input: I, terminator: Terminator) -> Self {
        Self {
            splitter: Splitter::new(input, terminator),
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
                Ok(path.into())
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

    #[test]
    fn position() {
        let mut position = Position::new();
        assert_eq!(position.to_string(), "item #1 at offset 0");
        position.increment(1);
        assert_eq!(position.to_string(), "item #2 at offset 1");
        position.increment(2);
        assert_eq!(position.to_string(), "item #3 at offset 3");
    }

    mod path_diff {
        use super::*;
        use crate::testing::unpack_io_error;
        use test_case::test_case;

        #[test_case("",                       0, None                 ; "empty")]
        #[test_case("<abc\n>def\n< g \n> h ", 0, Some(("abc", "def")) ; "nonempty 0")]
        #[test_case("<abc\n>def\n< g \n> h ", 1, Some((" g ", " h ")) ; "nonempty 1")]
        #[test_case("<abc\n>def\n< g \n> h ", 2, None                 ; "nonempty 2")]
        fn ok(input: &str, position: usize, result: Option<(&str, &str)>) {
            let mut path_diff =
                PathDiff::new(input.as_bytes(), Terminator::Newline { required: false });

            for _ in 0..position {
                path_diff.read().unwrap_or_default();
            }

            assert_eq!(
                path_diff.read().map_err(unpack_io_error),
                Ok(result.map(|(first, second)| (first.into(), second.into())))
            );
        }

        type E = ErrorKind;

        #[test_case("a",     E::InvalidData,   "Expected '<' but got 'a' (item #1 at offset 0)"  ; "in prefix invalid")]
        #[test_case("<",     E::UnexpectedEof, "Expected a path after '<' (item #1 at offset 0)" ; "in path missing")]
        #[test_case("<a",    E::UnexpectedEof, "Expected '>' (item #2 at offset 2)"              ; "in terminator missing")]
        #[test_case("<a\n",  E::UnexpectedEof, "Expected '>' (item #2 at offset 3)"              ; "out prefix missing")]
        #[test_case("<a\nb", E::InvalidData,   "Expected '>' but got 'b' (item #2 at offset 3)"  ; "out prefix invalid")]
        #[test_case("<a\n>", E::UnexpectedEof, "Expected a path after '>' (item #2 at offset 3)" ; "out path missing")]
        fn err(input: &str, kind: ErrorKind, message: &str) {
            assert_eq!(
                PathDiff::new(input.as_bytes(), Terminator::Newline { required: false })
                    .read()
                    .map_err(unpack_io_error),
                Err((kind, message.into()))
            )
        }
    }
}
