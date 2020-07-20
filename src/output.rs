use crate::pattern::Pattern;
use crate::utils::{highlight_range, spec_color, HasRange};
use std::fmt::Display;
use std::io::{Result, Write};
use termcolor::{Color, ColorChoice, StandardStream, WriteColor};

pub struct Output {
    stdout: StandardStream,
    stderr: StandardStream,
    delimiter: Option<char>,
}

impl Output {
    pub fn new(colors: ColorChoice, delimiter: Option<char>) -> Self {
        Self {
            stdout: StandardStream::stdout(colors), // TODO global lock
            stderr: StandardStream::stderr(colors), // TODO global lock
            delimiter,
        }
    }

    pub fn write_path(&mut self, path: &str) -> Result<()> {
        if let Some(delimiter) = self.delimiter {
            write!(self.stdout, "{}{}", path, delimiter)
        } else {
            write!(self.stdout, "{}", path)
        }
    }

    pub fn write_explanation(&mut self, pattern: &Pattern) -> Result<()> {
        pattern.explain(&mut self.stdout)
    }

    pub fn write_pattern_error<T: Display + HasRange>(
        &mut self,
        raw_pattern: &str,
        error: &T,
    ) -> Result<()> {
        self.write_error(&error)?;
        writeln!(self.stderr)?;
        highlight_range(&mut self.stderr, raw_pattern, error.range(), Color::Red)?;
        self.stderr.reset()
    }

    pub fn write_error<T: Display>(&mut self, error: &T) -> Result<()> {
        self.stderr.set_color(&spec_color(Color::Red))?;
        write!(self.stderr, "error:")?;
        self.stderr.reset()?;
        writeln!(self.stderr, " {}", error)
    }
}
