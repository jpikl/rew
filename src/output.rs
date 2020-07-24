use crate::utils::{highlight_range, spec_color, HasRange};
use std::error::Error;
use std::io::{Result, Write};
use termcolor::{Color, StandardStream, StandardStreamLock, WriteColor};

pub struct Paths<'a> {
    lock: StandardStreamLock<'a>,
    delimiter: Option<char>,
}

impl<'a> Paths<'a> {
    pub fn new(stream: &'a mut StandardStream, delimiter: Option<char>) -> Self {
        Self {
            lock: stream.lock(),
            delimiter,
        }
    }

    pub fn write(&mut self, path: &str) -> Result<()> {
        if let Some(delimiter) = self.delimiter {
            write!(self.lock, "{}{}", path, delimiter)
        } else {
            write!(self.lock, "{}", path)
        }
    }
}

pub struct Errors<'a> {
    lock: StandardStreamLock<'a>,
}

impl<'a> Errors<'a> {
    pub fn new(stream: &'a mut StandardStream) -> Self {
        Self {
            lock: stream.lock(),
        }
    }

    pub fn write<T: Error>(&mut self, error: &T) -> Result<()> {
        self.lock.set_color(&spec_color(Color::Red))?;
        write!(self.lock, "error:")?;
        self.lock.reset()?;
        writeln!(self.lock, " {}", error)
    }

    pub fn write_with_highlight<T: Error + HasRange>(
        &mut self,
        error: &T,
        raw_pattern: &str,
    ) -> Result<()> {
        self.write(error)?;
        writeln!(self.lock)?;
        highlight_range(&mut self.lock, raw_pattern, error.range(), Color::Red)?;
        self.lock.reset()
    }
}
