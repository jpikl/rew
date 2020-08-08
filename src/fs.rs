use std::path::Path;
use std::{error, fmt, result};

#[derive(Debug)]
pub struct Error {}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Not implemented!")
    }
}

impl error::Error for Error {}

pub type Result = result::Result<(), Error>;

pub struct Operations {
    overwrite: bool,
    recursive: bool,
}

impl Operations {
    pub fn new(overwrite: bool, recursive: bool) -> Self {
        Self {
            overwrite,
            recursive,
        }
    }

    pub fn rename_or_move(&self, source: &Path, target: &Path) -> Result {
        Err(Error {}) // TODO implement
    }

    pub fn copy(&self, source: &Path, target: &Path) -> Result {
        Err(Error {}) // TODO implement
    }
}
