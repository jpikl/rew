use common::input::{Delimiter, Splitter};
use std::io::{Result, StdinLock};
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Paths<'a> {
    Args { iter: Iter<'a, PathBuf> },
    Stdin { splitter: Splitter<StdinLock<'a>> },
}

impl<'a> Paths<'a> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Paths::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: StdinLock<'a>, delimiter: Delimiter) -> Self {
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
