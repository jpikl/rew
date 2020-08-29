use crate::fs;
use crate::output;
use std::path::Path;
use std::{io, result};
use termcolor::StandardStream;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Fs(fs::Error),
}

pub type Result = result::Result<(), Error>;

pub enum Action<'a> {
    Print {
        output: output::Paths<'a>,
    },
    PrettyPrint {
        output: output::PrettyPaths<'a>,
    },
    Move {
        fs: fs::Operations,
        output: output::Actions<'a>,
    },
    Copy {
        fs: fs::Operations,
        output: output::Actions<'a>,
    },
}

impl<'a> Action<'a> {
    pub fn print_paths(stream: &'a mut StandardStream, delimiter: Option<char>) -> Self {
        Self::Print {
            output: output::Paths::new(stream, delimiter),
        }
    }

    pub fn pretty_print_paths(stream: &'a mut StandardStream) -> Self {
        Self::PrettyPrint {
            output: output::PrettyPaths::new(stream),
        }
    }

    pub fn move_paths(stream: &'a mut StandardStream, overwrite: bool, recursive: bool) -> Self {
        Self::Move {
            fs: fs::Operations::new(overwrite, recursive),
            output: output::Actions::new(stream),
        }
    }

    pub fn copy_paths(stream: &'a mut StandardStream, overwrite: bool, recursive: bool) -> Self {
        Self::Copy {
            fs: fs::Operations::new(overwrite, recursive),
            output: output::Actions::new(stream),
        }
    }

    pub fn exec(&mut self, source: &Path, target: &str) -> Result {
        match self {
            Self::Print { output } => output.write(target).map_err(Error::Io),
            Self::PrettyPrint { output } => output.write(source, target).map_err(Error::Io),
            Self::Move { fs, output } => {
                let target = Path::new(&target);
                output.write_moving(source, target).map_err(Error::Io)?;
                match fs.rename_or_move(source, target) {
                    Ok(()) => output.write_success().map_err(Error::Io),
                    Err(error) => {
                        output.write_failure().map_err(Error::Io)?;
                        Err(Error::Fs(error))
                    }
                }
            }
            Self::Copy { fs, output } => {
                let target = Path::new(&target);
                output.write_copying(source, target).map_err(Error::Io)?;
                match fs.copy(source, target) {
                    Ok(()) => output.write_success().map_err(Error::Io),
                    Err(error) => {
                        output.write_failure().map_err(Error::Io)?;
                        Err(Error::Fs(error))
                    }
                }
            }
        }
    }
}
