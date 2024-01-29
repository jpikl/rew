use super::conf::LineConfig;
use super::conf::LineSeparator;
use super::write::Source;
use super::BufSizeConfig;
use anyhow::Result;
use derive_more::Display;
use derive_more::Error;
use memchr::memchr;
use std::io::stdin;
use std::io::Read;
use std::io::StdinLock;

pub struct BlockReader<R> {
    inner: R,
    buf: Vec<u8>,
}

impl BlockReader<StdinLock<'static>> {
    pub fn from_stdin(config: &impl BufSizeConfig) -> Self {
        Self::new(stdin().lock(), config.buf_size())
    }
}

impl<R: Read> BlockReader<R> {
    pub fn new(inner: R, buf_size: usize) -> Self {
        Self {
            inner,
            buf: vec![0; buf_size],
        }
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

impl<R: Read> Source for BlockReader<R> {
    type Reader = R;

    fn reader(&mut self) -> &mut Self::Reader {
        &mut self.inner
    }
}

#[derive(Debug, Display, Error, PartialEq)]
#[display("cannot process input line bigger than '{_0}' bytes")]
struct MaxLineError(#[error(not(source))] usize);

pub trait LineReaderConfig {
    fn line_buf_size(&self) -> usize;
}

pub struct LineReader<R> {
    inner: R,
    separator: u8,
    trim: fn(&[u8]) -> &[u8],
    buf: Vec<u8>,
    start: usize,
    end: usize,
}

impl LineReader<StdinLock<'static>> {
    pub fn from_stdin(config: &(impl LineConfig + BufSizeConfig)) -> Self {
        Self::new(stdin().lock(), config.line_separator(), config.buf_size())
    }
}

impl<R: Read> LineReader<R> {
    pub fn new(inner: R, separator: LineSeparator, buf_size: usize) -> Self {
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
                return Ok(Some(self.empty_buf()));
            }

            // Input line could not fit into the whole buffer
            return Err(MaxLineError(self.buf.len()).into());
        }
    }
}

impl<R: Read> Source for LineReader<R> {
    type Reader = R;

    fn empty_buf(&mut self) -> &[u8] {
        let result = &self.buf[self.start..self.end];
        self.start = 0;
        self.end = 0;
        result
    }

    fn reader(&mut self) -> &mut Self::Reader {
        &mut self.inner
    }
}
