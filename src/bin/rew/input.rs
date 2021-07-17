use std::io::{BufRead, Result};
use std::slice::Iter;

use common::input::{Splitter, Terminator};

pub enum Values<'a, A: AsRef<str>, I: BufRead> {
    Args { iter: Iter<'a, A> },
    Stdin { splitter: Splitter<I> },
}

impl<'a, A: AsRef<str>, I: BufRead> Values<'a, A, I> {
    pub fn from_args(values: &'a [A]) -> Self {
        Values::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: I, terminator: Terminator) -> Self {
        Values::Stdin {
            splitter: Splitter::new(stdin, terminator),
        }
    }

    pub fn next(&mut self) -> Result<Option<&str>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(A::as_ref)),
            Self::Stdin { splitter: reader } => Ok(reader.read()?.map(|(value, _)| value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use common::testing::unpack_io_error;
    use test_case::test_case;

    use super::*;

    #[test_case(args(),  0, Some("a") ; "args 0")]
    #[test_case(args(),  1, Some("b") ; "args 1")]
    #[test_case(args(),  2, None      ; "args 2")]
    #[test_case(stdin(), 0, Some("a") ; "stdin 0")]
    #[test_case(stdin(), 1, Some("b") ; "stdin 1")]
    #[test_case(stdin(), 2, None      ; "stdin 2")]
    fn next(mut values: Values<&str, &[u8]>, position: usize, result: Option<&str>) {
        for _ in 0..position {
            values.next().unwrap_or_default();
        }
        assert_eq!(values.next().map_err(unpack_io_error), Ok(result));
    }

    fn args<'a>() -> Values<'a, &'a str, &'a [u8]> {
        Values::from_args(&["a", "b"][..])
    }

    fn stdin<'a>() -> Values<'a, &'a str, &'a [u8]> {
        Values::from_stdin(&b"a\nb"[..], Terminator::Newline { required: false })
    }
}
