use crate::input::Input;
use clap::OsValues;
use std::io::Result;
use std::path::Path;

pub struct ArgsInput<'a> {
    values: OsValues<'a>,
}

impl<'a> ArgsInput<'a> {
    pub fn new(values: OsValues<'a>) -> Self {
        Self { values }
    }
}

impl<'a> Input for ArgsInput<'a> {
    fn next(&mut self) -> Result<Option<&Path>> {
        Ok(self.values.next().map(Path::new))
    }
}
