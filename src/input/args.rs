use crate::input::Input;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub struct ArgsInput<'a> {
    iter: Iter<'a, PathBuf>,
}

impl<'a> ArgsInput<'a> {
    pub fn new(values: &'a [PathBuf]) -> Self {
        Self {
            iter: values.iter(),
        }
    }
}

impl<'a> Input for ArgsInput<'a> {
    fn next(&mut self) -> Result<Option<&Path>> {
        Ok(self.iter.next().map(PathBuf::as_path))
    }
}
