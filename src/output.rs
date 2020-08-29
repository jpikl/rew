use crate::utils::{highlight_range, spec_color, HasRange};
use std::error::Error;
use std::io::{Result, Write};
use std::path::Path;
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
            write!(self.lock, "{}{}", path, delimiter)?;
            if delimiter != '\n' {
                self.lock.flush()?;
            }
            Ok(())
        } else {
            write!(self.lock, "{}", path)?;
            self.lock.flush()
        }
    }
}

pub struct PrettyPaths<'a> {
    lock: StandardStreamLock<'a>,
}

impl<'a> PrettyPaths<'a> {
    pub fn new(stream: &'a mut StandardStream) -> Self {
        Self {
            lock: stream.lock(),
        }
    }

    pub fn write(&mut self, source: &Path, target: &str) -> Result<()> {
        self.lock.set_color(&spec_color(Color::Blue))?;
        write!(self.lock, "{}", source.to_string_lossy())?;
        self.lock.reset()?;
        write!(self.lock, " -> ")?;
        self.lock.set_color(&spec_color(Color::Green))?;
        writeln!(self.lock, "{}", target)?;
        self.lock.reset()
    }
}

pub struct Actions<'a> {
    lock: StandardStreamLock<'a>,
}

impl<'a> Actions<'a> {
    pub fn new(stream: &'a mut StandardStream) -> Self {
        Self {
            lock: stream.lock(),
        }
    }

    pub fn write_moving(&mut self, source: &Path, target: &Path) -> Result<()> {
        self.write("Moving", source, target)
    }

    pub fn write_copying(&mut self, source: &Path, target: &Path) -> Result<()> {
        self.write("Copying", source, target)
    }

    fn write(&mut self, action: &str, source: &Path, target: &Path) -> Result<()> {
        write!(self.lock, "{} '", action)?;
        self.lock.set_color(&spec_color(Color::Blue))?;
        write!(self.lock, "{}", source.to_string_lossy())?;
        self.lock.reset()?;
        write!(self.lock, "' to '")?;
        self.lock.set_color(&spec_color(Color::Blue))?;
        write!(self.lock, "{}", target.to_string_lossy())?;
        self.lock.reset()?;
        write!(self.lock, "' ... ")?;
        self.lock.flush()
    }

    pub fn write_success(&mut self) -> Result<()> {
        self.lock.set_color(&spec_color(Color::Green))?;
        writeln!(self.lock, "OK")?;
        self.lock.reset()
    }

    pub fn write_failure(&mut self) -> Result<()> {
        self.lock.set_color(&spec_color(Color::Red))?;
        writeln!(self.lock, "FAILED")?;
        self.lock.reset()
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
