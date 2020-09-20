use common::input::{Delimiter, Splitter};
use std::io::{BufRead, Result};
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Paths<'a, T: BufRead> {
    Args { iter: Iter<'a, PathBuf> },
    Stdin { splitter: Splitter<T> },
}

impl<'a, T: BufRead> Paths<'a, T> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Paths::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: T, delimiter: Delimiter) -> Self {
        Paths::Stdin {
            splitter: Splitter::new(stdin, delimiter),
        }
    }

    pub fn next(&mut self) -> Result<Option<&Path>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(PathBuf::as_path)),
            Self::Stdin { splitter: reader } => reader.read().map(|opt_str| opt_str.map(Path::new)),
        }
    }
}
