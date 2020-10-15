use common::input::{Delimiter, Splitter};
use common::io::Input;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Paths<'a, I: Input> {
    Args { iter: Iter<'a, PathBuf> },
    Stdin { splitter: Splitter<I> },
}

impl<'a, I: Input> Paths<'a, I> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Paths::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(input: I, delimiter: Delimiter) -> Self {
        Paths::Stdin {
            splitter: Splitter::new(input, delimiter),
        }
    }

    pub fn next(&mut self) -> Result<Option<&Path>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(PathBuf::as_path)),
            Self::Stdin { splitter: reader } => reader.read().map(|opt_str| opt_str.map(Path::new)),
        }
    }
}
