use crate::utils::{highlight_range, HasRange};
use common::color::spec_color;
use common::io::Output;
use common::output::write_error;
use std::error::Error;
use std::io::Result;
use std::path::Path;
use termcolor::Color;

pub enum PathMode {
    Out(Option<char>),
    InOut(Option<char>),
    InOutPretty,
}

pub struct Paths<O: Output> {
    output: O,
    mode: PathMode,
}

impl<O: Output> Paths<O> {
    pub fn new(output: O, mode: PathMode) -> Self {
        Self { output, mode }
    }

    pub fn write(&mut self, input_path: &Path, output_path: &str) -> Result<()> {
        match self.mode {
            PathMode::Out(Some(delimiter)) => {
                write!(self.output, "{}{}", output_path, delimiter)?;
                if delimiter != '\n' {
                    self.output.flush()
                } else {
                    Ok(())
                }
            }
            PathMode::Out(None) => {
                write!(self.output, "{}", output_path)?;
                self.output.flush()
            }
            PathMode::InOut(Some(delimiter)) => {
                write!(
                    self.output,
                    "<{}{}>{}{}",
                    input_path.to_string_lossy(),
                    delimiter,
                    output_path,
                    delimiter
                )?;
                if delimiter != '\n' {
                    self.output.flush()
                } else {
                    Ok(())
                }
            }
            PathMode::InOut(None) => {
                write!(
                    self.output,
                    "<{}>{}",
                    input_path.to_string_lossy(),
                    output_path
                )?;
                self.output.flush()
            }
            PathMode::InOutPretty => {
                self.output.set_color(&spec_color(Color::Blue))?;
                write!(self.output, "{}", input_path.to_string_lossy())?;
                self.output.reset()?;
                write!(self.output, " -> ")?;
                self.output.set_color(&spec_color(Color::Green))?;
                writeln!(self.output, "{}", output_path)
            }
        }
    }
}

pub fn write_pattern_error<O: Output, E: Error + HasRange>(
    output: &mut O,
    error: &E,
    raw_pattern: &str,
) -> Result<()> {
    write_error(output, error)?;
    writeln!(output)?;
    highlight_range(output, raw_pattern, error.range(), Color::Red)?;
    output.reset()
}
