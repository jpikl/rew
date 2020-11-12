use common::input::{Delimiter, Splitter};
use std::io::{BufRead, Result};
use std::slice::Iter;

pub enum Paths<'a, I: BufRead> {
    Args { iter: Iter<'a, String> },
    Stdin { splitter: Splitter<I> },
}

impl<'a, I: BufRead> Paths<'a, I> {
    pub fn from_args(values: &'a [String]) -> Self {
        Paths::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: I, delimiter: Delimiter) -> Self {
        Paths::Stdin {
            splitter: Splitter::new(stdin, delimiter),
        }
    }

    pub fn next(&mut self) -> Result<Option<&str>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(String::as_str)),
            Self::Stdin { splitter: reader } => match reader.read() {
                Ok(Some((value, _))) => Ok(Some(value)),
                Ok(None) => Ok(None),
                Err(error) => Err(error),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::testing::unpack_io_error;

    #[test]
    fn paths_from_args() {
        let args = vec![String::from("a"), String::from("b")];
        let mut paths: Paths<&[u8]> = Paths::from_args(&args);
        assert_eq!(paths.next().map_err(unpack_io_error), Ok(Some("a")));
        assert_eq!(paths.next().map_err(unpack_io_error), Ok(Some("b")));
        assert_eq!(paths.next().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn paths_from_stdin() {
        let mut paths = Paths::from_stdin(&b"a\nb"[..], Delimiter::Newline);
        assert_eq!(paths.next().map_err(unpack_io_error), Ok(Some("a")));
        assert_eq!(paths.next().map_err(unpack_io_error), Ok(Some("b")));
        assert_eq!(paths.next().map_err(unpack_io_error), Ok(None));
    }
}
