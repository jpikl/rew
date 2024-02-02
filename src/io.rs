use anyhow::anyhow;
use anyhow::Result;
use bstr::ByteSlice;
use memchr::memchr;
use std::io::copy;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

#[derive(Copy, Clone)]
pub enum Separator {
    Newline,
    Null,
}

impl Separator {
    pub fn as_byte(self) -> u8 {
        match self {
            Self::Newline => b'\n',
            Self::Null => b'\0',
        }
    }

    pub fn trim_fn(self) -> fn(&[u8]) -> &[u8] {
        match self {
            Self::Newline => trim_newline,
            Self::Null => trim_null,
        }
    }
}

fn trim_newline(mut line: &[u8]) -> &[u8] {
    if line.last_byte() == Some(b'\n') {
        line = &line[..line.len() - 1];
        if line.last_byte() == Some(b'\r') {
            line = &line[..line.len() - 1];
        }
    }
    line
}

fn trim_null(mut line: &[u8]) -> &[u8] {
    if line.last_byte() == Some(b'\0') {
        line = &line[..line.len() - 1];
    }
    line
}

pub struct BlockReader<R> {
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> BlockReader<R> {
    pub fn new(inner: R, buf_size: usize) -> Self {
        Self {
            inner,
            buf: vec![0; buf_size],
        }
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn read_block(&mut self) -> Result<Option<&mut [u8]>> {
        let len = self.inner.read(&mut self.buf)?;
        if len > 0 {
            Ok(Some(&mut self.buf[..len]))
        } else {
            Ok(None)
        }
    }
}

pub struct LineReader<R> {
    inner: R,
    separator: u8,
    trim: fn(&[u8]) -> &[u8],
    buf: Vec<u8>,
    start: usize,
    end: usize,
}

impl<R: Read> LineReader<R> {
    pub fn new(inner: R, separator: Separator, buf_size: usize) -> Self {
        Self {
            inner,
            separator: separator.as_byte(),
            trim: separator.trim_fn(),
            buf: vec![0; buf_size],
            start: 0,
            end: 0,
        }
    }

    pub fn read_line(&mut self) -> Result<Option<&[u8]>> {
        if self.end == 0 {
            // Re-fill buffer after it's consumption
            let len = self.inner.read(&mut self.buf)?;
            if len == 0 {
                return Ok(None); // End of input
            }
            self.end += len;
        }

        loop {
            // Find the next line ending in unprocessed buffer area
            if let Some(pos) = memchr(self.separator, &self.buf[self.start..self.end]) {
                let line_end = self.start + pos + 1;
                let line = &self.buf[self.start..line_end];

                if line_end == self.end {
                    self.start = 0;
                    self.end = 0;
                } else {
                    self.start = line_end;
                }

                return Ok(Some((self.trim)(line)));
            }

            if self.start > 0 {
                // Compact data in the buffer to get more free space at its end
                self.buf.copy_within(self.start..self.end, 0);
                self.end -= self.start;
                self.start = 0;
            }

            if self.end < self.buf.len() {
                // Try to fetch more data into the buffer to complete a line
                let len = self.inner.read(&mut self.buf[self.end..])?;
                if len > 0 {
                    self.end += len;
                    continue; // Re-try line ending detection
                }
                // End of input => output unterminated line in the buffer
                let remainder = &self.buf[self.start..self.end];
                self.start = 0;
                self.end = 0;
                return Ok(Some(remainder));
            }

            // Input line could not fit into the whole buffer
            return Err(anyhow!(
                "cannot fetch line longer than '{}' bytes",
                self.buf.len()
            ));
        }
    }
}

pub struct Writer<W: Write> {
    inner: BufWriter<W>,
    separator: u8,
    buffered: bool,
}

impl<W: Write> Writer<W> {
    pub fn new(inner: W, separator: Separator, buffered: bool, buf_size: usize) -> Self {
        Self {
            inner: BufWriter::with_capacity(buf_size, inner),
            separator: separator.as_byte(),
            buffered,
        }
    }

    pub fn write_line(&mut self, line: &[u8]) -> Result<()> {
        if self.buffered {
            self.inner.write_all(line)?;
            self.inner.write_all(&[self.separator])?;
        } else {
            self.inner.get_mut().write_all(line)?;
            self.inner.get_mut().write_all(&[self.separator])?;
        }
        Ok(())
    }

    pub fn write_block(&mut self, block: &[u8]) -> Result<()> {
        if self.buffered {
            self.inner.write_all(block)?;
        } else {
            self.inner.get_mut().write_all(block)?;
        }
        Ok(())
    }

    pub fn write_all_from(&mut self, reader: &mut impl Read) -> Result<()> {
        if self.buffered {
            copy(reader, &mut self.inner)?;
        } else {
            copy(reader, &mut self.inner.get_mut())?;
        }
        Ok(())
    }
}