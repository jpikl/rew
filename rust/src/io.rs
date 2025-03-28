use anyhow::format_err;
use anyhow::Result;
use bstr::ByteSlice;
use derive_more::IsVariant;
use memchr::memchr;
use std::io::Read;

#[derive(Copy, Clone, IsVariant)]
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

pub struct LineReader<R> {
    inner: R,
    separator: u8,
    trim: fn(&[u8]) -> &[u8],
    buf: Vec<u8>,
    start: usize, // Start offset of unprocessed buf area
    end: usize,   // End offset of unprocessed buf area
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
        loop {
            // Find the next line ending in unprocessed buffer area
            if let Some(pos) = memchr(self.separator, &self.buf[self.start..self.end]) {
                let line = &self.buf[self.start..][..(pos + 1)];
                self.start += pos + 1;
                return Ok(Some((self.trim)(line)));
            }

            if self.start > 0 {
                // Compact data in the buffer to get more free space at its end
                self.buf.copy_within(self.start..self.end, 0);
                self.end -= self.start;
                self.start = 0;
            }

            // Try to fetch more data into the buffer to complete a line
            let len = self.inner.read(&mut self.buf[self.end..])?;
            if len > 0 {
                self.end += len;
                continue; // Re-try line ending detection
            }

            // End of input (and the buffer is empty)
            if self.end == 0 {
                return Ok(None);
            }

            // End of input => output unterminated line in the buffer
            if self.end < self.buf.len() {
                let remainder = &self.buf[self.start..self.end];
                self.start = self.end;
                return Ok(Some(remainder));
            }

            // Input line could not fit into the whole buffer
            return Err(format_err!(
                "could not fetch line longer than '{}' bytes",
                self.buf.len()
            ));
        }
    }
}
