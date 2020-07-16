use crate::pattern::{eval, parse, Pattern};
use crate::utils::{highlight_range, spec_color};
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
        if let Some(delimiter_value) = self.delimiter {
            write!(self.stdout, "{}{}", path, delimiter_value)
        } else {
            write!(self.stdout, "{}", path)
        }
    }

    pub fn write_explanation(&mut self, pattern: &Pattern, raw_pattern: &str) -> Result<()> {
        pattern.explain(&mut self.stdout, raw_pattern)
    }

    pub fn write_parse_error(&mut self, raw_pattern: &str, error: &parse::Error) -> Result<()> {
        self.write_error(&error)?;
        writeln!(self.stderr)?;
        highlight_range(&mut self.stderr, raw_pattern, &error.range, Color::Red)?;
        self.stderr.reset()
    }

    pub fn write_eval_error(&mut self, raw_pattern: &str, error: &eval::Error) -> Result<()> {
        self.write_error(&error)?;
        writeln!(self.stderr)?;
        highlight_range(
            &mut self.stderr,
            raw_pattern,
            &error.cause.range(),
            Color::Red,
        )?;
        self.stderr.reset()
    }

    pub fn write_error<T: Display>(&mut self, error: &T) -> Result<()> {
        self.stderr.set_color(&spec_color(Color::Red))?;
        write!(self.stderr, "error:")?;
        self.stderr.reset()?;
        writeln!(self.stderr, " {}", error)
    }
}
