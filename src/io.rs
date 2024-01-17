use anyhow::Result;
use bstr::ByteSlice;
use clap::ValueEnum;
use linereader::LineReader;
use memchr::memchr;
use memchr::memrchr;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::io::BufReader;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use thiserror::Error;

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library.
pub const OPTIMAL_IO_BUF_SIZE: usize = 32 * 1024;

#[derive(Debug, Error, PartialEq)]
#[error("cannot process input line bigger than '{}' bytes", .0)]
struct MaxLineError(usize);

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum Separator {
    #[default]
    Newline,
    Null,
}

impl Separator {
    fn as_byte(&self) -> u8 {
        match self {
            Self::Newline => b'\n',
            Self::Null => b'\0',
        }
    }

    fn trim_end<'a>(&self, mut line: &'a [u8]) -> &'a [u8] {
        match self {
            Self::Newline => {
                if line.last_byte() == Some(b'\n') {
                    line = &line[..line.len() - 1];
                    if line.last_byte() == Some(b'\r') {
                        line = &line[..line.len() - 1];
                    }
                }
            }
            Self::Null => {
                if line.last_byte() == Some(b'\0') {
                    line = &line[..line.len() - 1];
                }
            }
        }
        line
    }
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum Buffering {
    Line,
    Full,
}

impl Display for Buffering {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Line => write!(fmt, "line"),
            Self::Full => write!(fmt, "full"),
        }
    }
}

impl Default for Buffering {
    fn default() -> Self {
        if io::stdout().is_terminal() {
            Self::Line
        } else {
            Self::Full
        }
    }
}

pub enum Processing {
    Continue,
    Abort,
}

pub struct Reader<R> {
    inner: R,
    separator: Separator,
    max_line: usize,
}

impl<R: Read> Reader<R> {
    pub fn new(inner: R, separator: Separator, max_line: usize) -> Self {
        Self {
            inner,
            separator,
            max_line,
        }
    }

    pub fn for_each_line<F: FnMut(&[u8]) -> Result<Processing>>(
        &mut self,
        mut action: F,
    ) -> Result<()> {
        let byte_separator = self.separator.as_byte();
        let mut reader =
            LineReader::with_delimiter_and_capacity(byte_separator, self.max_line, &mut self.inner);

        while let Some(batch) = reader.next_batch() {
            let mut batch = batch?;

            if batch.len() == self.max_line && memrchr(byte_separator, batch).is_none() {
                return Err(MaxLineError(self.max_line).into());
            }

            while let Some(end) = memchr(byte_separator, batch) {
                let (line, next_batch) = batch.split_at(end + 1);

                match action(self.separator.trim_end(line)) {
                    Ok(Processing::Continue) => {}
                    Ok(Processing::Abort) => return Ok(()),
                    Err(err) => return Err(err),
                }

                batch = next_batch;
            }

            if !batch.is_empty() {
                match action(self.separator.trim_end(batch)) {
                    Ok(Processing::Continue) => {}
                    Ok(Processing::Abort) => return Ok(()),
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(())
    }

    pub fn for_each_block<F: FnMut(&[u8]) -> Result<Processing>>(
        &mut self,
        mut action: F,
    ) -> Result<()> {
        let mut reader = BufReader::new(&mut self.inner);
        let mut buffer = vec![0; OPTIMAL_IO_BUF_SIZE];

        loop {
            let len = reader.read(&mut buffer)?;
            if len == 0 {
                break;
            }
            match action(&buffer[..len]) {
                Ok(Processing::Continue) => {}
                Ok(Processing::Abort) => return Ok(()),
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

pub struct Writer<W> {
    inner: W,
    separator: u8,
    buffering: Buffering,
}

impl<W: Write> Writer<W> {
    pub fn new(inner: W, separator: Separator, buffering: Buffering) -> Self {
        Self {
            inner,
            separator: separator.as_byte(),
            buffering,
        }
    }

    pub fn write_line(&mut self, line: &[u8]) -> Result<()> {
        self.inner.write_all(line)?;
        self.inner.write_all(&[self.separator])?;

        match self.buffering {
            Buffering::Line => self.inner.flush().map_err(Into::into),
            Buffering::Full => Ok(()),
        }
    }

    pub fn write_block(&mut self, block: &[u8]) -> Result<()> {
        match self.buffering {
            Buffering::Line => {
                // We do not care much about the performance in this mode
                if let Some(pos) = memrchr(self.separator, block) {
                    let (before, after) = block.split_at(pos + 1);
                    self.inner.write_all(before)?;
                    self.inner.flush()?;
                    self.inner.write_all(after)?;
                } else {
                    self.inner.write_all(block)?;
                }
            }
            Buffering::Full => self.inner.write_all(block)?,
        }
        Ok(())
    }
}
