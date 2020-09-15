use crate::utils::{highlight_range, HasRange};
use common::color::spec_color;
use common::output::write_error;
use std::error::Error;
use std::io::{Result, Write};
use std::path::Path;
use termcolor::{Color, WriteColor};

pub enum PathMode {
    Out(Option<char>),
    InOut(Option<char>),
    InOutPretty,
}

pub struct Paths<S: Write + WriteColor> {
    stream: S,
    mode: PathMode,
}

impl<S: Write + WriteColor> Paths<S> {
    pub fn new(stream: S, mode: PathMode) -> Self {
        Self { stream, mode }
    }

    pub fn write(&mut self, input_path: &Path, output_path: &str) -> Result<()> {
        match self.mode {
            PathMode::Out(Some(delimiter)) => {
                write!(self.stream, "{}{}", output_path, delimiter)?;
                if delimiter != '\n' {
                    self.stream.flush()
                } else {
                    Ok(())
                }
            }
            PathMode::Out(None) => {
                write!(self.stream, "{}", output_path)?;
                self.stream.flush()
            }
            PathMode::InOut(Some(delimiter)) => {
                write!(
                    self.stream,
                    "<{}{}>{}{}",
                    input_path.to_string_lossy(),
                    delimiter,
                    output_path,
                    delimiter
                )?;
                if delimiter != '\n' {
                    self.stream.flush()
                } else {
                    Ok(())
                }
            }
            PathMode::InOut(None) => {
                write!(
                    self.stream,
                    "<{}>{}",
                    input_path.to_string_lossy(),
                    output_path
                )?;
                self.stream.flush()
            }
            PathMode::InOutPretty => {
                self.stream.set_color(&spec_color(Color::Blue))?;
                write!(self.stream, "{}", input_path.to_string_lossy())?;
                self.stream.reset()?;
                write!(self.stream, " -> ")?;
                self.stream.set_color(&spec_color(Color::Green))?;
                writeln!(self.stream, "{}", output_path)
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
