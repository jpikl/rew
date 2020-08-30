use crate::utils::{highlight_range, HasRange};
use common::{spec_color, write_error};
use std::error::Error;
use std::io::{Result, Write};
use std::path::Path;
use termcolor::{Color, StandardStream, StandardStreamLock, WriteColor};

pub enum PathMode {
    Out(Option<char>),
    InOut(Option<char>),
    InOutPretty,
}

pub struct Paths<'a> {
    lock: StandardStreamLock<'a>,
    mode: PathMode,
}

impl<'a> Paths<'a> {
    pub fn new(stream: &'a mut StandardStream, mode: PathMode) -> Self {
        Self {
            lock: stream.lock(),
            mode,
        }
    }

    pub fn write(&mut self, input_path: &Path, output_path: &str) -> Result<()> {
        match self.mode {
            PathMode::Out(Some(delimiter)) => {
                write!(self.lock, "{}{}", output_path, delimiter)?;
                if delimiter != '\n' {
                    self.lock.flush()
                } else {
                    Ok(())
                }
            }
            PathMode::Out(None) => {
                write!(self.lock, "{}", output_path)?;
                self.lock.flush()
            }
            PathMode::InOut(Some(delimiter)) => {
                write!(
                    self.lock,
                    "<{}{}>{}{}",
                    input_path.to_string_lossy(),
                    delimiter,
                    output_path,
                    delimiter
                )?;
                if delimiter != '\n' {
                    self.lock.flush()
                } else {
                    Ok(())
                }
            }
            PathMode::InOut(None) => {
                write!(
                    self.lock,
                    "<{}>{}",
                    input_path.to_string_lossy(),
                    output_path
                )?;
                self.lock.flush()
            }
            PathMode::InOutPretty => {
                self.lock.set_color(&spec_color(Color::Blue))?;
                write!(self.lock, "{}", input_path.to_string_lossy())?;
                self.lock.reset()?;
                write!(self.lock, " -> ")?;
                self.lock.set_color(&spec_color(Color::Green))?;
                writeln!(self.lock, "{}", output_path)
            }
        }
    }
}

pub fn write_pattern_error<S: Write + WriteColor, E: Error + HasRange>(
    stream: &mut S,
    error: &E,
    raw_pattern: &str,
) -> Result<()> {
    write_error(stream, error)?;
    writeln!(stream)?;
    highlight_range(stream, raw_pattern, error.range(), Color::Red)?;
    stream.reset()
}
