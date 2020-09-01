use common::input::{Delimiter, Reader};
use std::io::{Result, Stdin, StdinLock};
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Paths<'a> {
    Args { iter: Iter<'a, PathBuf> },
    Stdin { reader: Reader<StdinLock<'a>> },
}

impl<'a> Paths<'a> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Paths::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: &'a mut Stdin, delimiter: Delimiter) -> Self {
        Paths::Stdin {
            reader: Reader::new(stdin.lock(), delimiter),
        }
    }

    pub fn next(&mut self) -> Result<Option<&Path>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(PathBuf::as_path)),
            Self::Stdin { reader } => reader.read().map(|opt_str| opt_str.map(Path::new)),
        }
    }
}
