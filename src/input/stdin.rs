use crate::input::Input;
use std::io::{BufRead, Error, ErrorKind, Result, Stdin, StdinLock};
use std::path::Path;

pub struct StdinInput<'a> {
    buf: Vec<u8>,
    guard: StdinLock<'a>,
    delimiter: u8,
}

impl<'a> StdinInput<'a> {
    pub fn new(stdin: &'a mut Stdin, null_delimited: bool) -> Self {
        Self {
            buf: Vec::new(),
            guard: stdin.lock(),
            delimiter: if null_delimited { 0 } else { b'\n' },
        }
    }
}

impl<'a> Input for StdinInput<'a> {
    fn next(&mut self) -> Result<Option<&Path>> {
        self.buf.clear();
        match self.guard.read_until(self.delimiter, &mut self.buf) {
            Ok(0) => Ok(None),
            Ok(mut size) => {
                if self.buf[size - 1] == self.delimiter {
                    size -= 1;
                    if self.delimiter == b'\n' && size > 0 && self.buf[size - 1] == b'\r' {
                        size -= 1;
                    }
                }
                match std::str::from_utf8(&self.buf[..size]) {
                    Ok(str) => Ok(Some(Path::new(str))),
                    Err(error) => Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Input does not have UTF-8 encoding (offset: {})",
                            error.valid_up_to()
                        ),
                    )),
                }
            }
            Err(e) => Err(e),
        }
    }
}
