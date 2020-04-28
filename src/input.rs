use std::io::{BufRead, Error, ErrorKind, Result, Stdin, StdinLock};
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Input<'a> {
    Args {
        iter: Iter<'a, PathBuf>,
    },
    Stdin {
        buffer: Vec<u8>,
        guard: StdinLock<'a>,
        delimiter: u8,
    },
}

impl<'a> Input<'a> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Input::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: &'a mut Stdin, delimiter: u8) -> Self {
        Input::Stdin {
            buffer: Vec::new(),
            guard: stdin.lock(),
            delimiter,
        }
    }

    pub fn next(&mut self) -> Result<Option<&Path>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(PathBuf::as_path)),
            Self::Stdin {
                buffer,
                guard,
                delimiter,
            } => {
                buffer.clear();
                match guard.read_until(*delimiter, buffer) {
                    Ok(0) => Ok(None),
                    Ok(mut size) => {
                        if buffer[size - 1] == *delimiter {
                            size -= 1;
                            if *delimiter == b'\n' && size > 0 && buffer[size - 1] == b'\r' {
                                size -= 1;
                            }
                        }
                        match std::str::from_utf8(&buffer[..size]) {
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
    }
}
