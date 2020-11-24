use common::input::{Delimiter, Splitter};
use std::io::{BufRead, Result};
use std::slice::Iter;

pub enum Values<'a, I: BufRead> {
    Args { iter: Iter<'a, String> },
    Stdin { splitter: Splitter<I> },
}

impl<'a, I: BufRead> Values<'a, I> {
    pub fn from_args(values: &'a [String]) -> Self {
        Values::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: I, delimiter: Delimiter) -> Self {
        Values::Stdin {
            splitter: Splitter::new(stdin, delimiter),
        }
    }

    pub fn next(&mut self) -> Result<Option<&str>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(String::as_str)),
            Self::Stdin { splitter: reader } => Ok(reader.read()?.map(|(value, _)| value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::testing::unpack_io_error;

    #[test]
    fn args() {
        let args = vec![String::from("a"), String::from("b")];
        let mut values: Values<&[u8]> = Values::from_args(&args);
        assert_eq!(values.next().map_err(unpack_io_error), Ok(Some("a")));
        assert_eq!(values.next().map_err(unpack_io_error), Ok(Some("b")));
        assert_eq!(values.next().map_err(unpack_io_error), Ok(None));
    }

    #[test]
    fn stdin() {
        let mut values = Values::from_stdin(&b"a\nb"[..], Delimiter::Newline);
        assert_eq!(values.next().map_err(unpack_io_error), Ok(Some("a")));
        assert_eq!(values.next().map_err(unpack_io_error), Ok(Some("b")));
        assert_eq!(values.next().map_err(unpack_io_error), Ok(None));
    }
}
