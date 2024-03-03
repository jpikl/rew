use anyhow::format_err;
use anyhow::Result;
use bstr::decode_last_utf8;
use bstr::ByteSlice;
use derive_more::IsVariant;
use memchr::memchr;
use std::io::copy;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

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

pub struct ByteChunkReader<R> {
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> ByteChunkReader<R> {
    pub fn new(inner: R, buf_size: usize) -> Self {
        Self {
            inner,
            buf: vec![0; buf_size],
        }
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn read_chunk(&mut self) -> Result<Option<&mut [u8]>> {
        let len = self.inner.read(&mut self.buf)?;
        if len > 0 {
            Ok(Some(&mut self.buf[..len]))
        } else {
            Ok(None)
        }
    }
}

pub struct CharChunkReader<R> {
    inner: R,
    buf: Vec<u8>,
    start: usize, // Start offset of unprocessed buf area
    end: usize,   // End offset of unprocessed buf area
}

impl<R: Read> CharChunkReader<R> {
    pub fn new(inner: R, buf_size: usize) -> Self {
        Self {
            inner,
            buf: vec![0; buf_size],
            start: 0,
            end: 0,
        }
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn read_chunk(&mut self) -> Result<Option<&mut [u8]>> {
        if self.start > 0 {
            // Shift unprocessed remainder to the beginning
            self.buf.copy_within(self.start..self.end, 0);
            self.end -= self.start;
            self.start = 0;
        };

        // Read new data after the previous remainder
        let len = self.inner.read(&mut self.buf[self.end..])?;
        if len > 0 {
            self.end += len;

            if let (None, remainder) = decode_last_utf8(&self.buf[..self.end]) {
                // The new non-utf-8 remainder will be processed the next read
                self.start = self.end - remainder;
                if self.start > 0 {
                    return Ok(Some(&mut self.buf[..self.start]));
                }
                return Err(format_err!(
                    "could not fetch utf-8 character longer than '{}' bytes",
                    self.buf.len()
                ));
            }
        }

        if self.end > 0 {
            let remainder = &mut self.buf[..self.end];
            self.end = 0;
            return Ok(Some(remainder));
        }

        Ok(None)
    }
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
            return Err(format_err!(
                "could not fetch line longer than '{}' bytes",
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
        self.write(line)?;
        self.write_separator()
    }

    pub fn write_separator(&mut self) -> Result<()> {
        self.write(&[self.separator])
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<()> {
        if self.buffered {
            self.inner.write_all(buf)?;
        } else {
            self.inner.get_mut().write_all(buf)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bstr::B;
    use claims::assert_err;
    use claims::assert_ok;
    use claims::assert_ok_eq;
    use claims::assert_some_eq;
    use rstest::rstest;

    #[test]
    fn read_byte_chunks() {
        let input = B("abcdefghijkl");
        let mut reader = ByteChunkReader::new(input, 8);
        assert_some_eq!(assert_ok!(reader.read_chunk()), B("abcdefgh"));
        assert_some_eq!(assert_ok!(reader.read_chunk()), B("ijkl"));
        assert_ok_eq!(reader.read_chunk(), None);
    }

    #[test]
    fn read_char_chunks() {
        let input = B("aábácádáeáfá");
        let input = &input[..(input.len() - 1)]; // Make the last byte invalid utf-8
        let mut reader = CharChunkReader::new(input, 8);
        assert_some_eq!(assert_ok!(reader.read_chunk()), B("aábác")); // 7B
        assert_some_eq!(assert_ok!(reader.read_chunk()), B("ádáeá")); // 8B
        assert_some_eq!(assert_ok!(reader.read_chunk()), B("f")); // 1B
        assert_some_eq!(assert_ok!(reader.read_chunk()), &B("á")[0..1]); // 1B
        assert_ok_eq!(reader.read_chunk(), None);
    }

    #[rstest]
    #[case::lf("abcd\nefgh\nijkl", Separator::Newline)]
    #[case::crlf("abcd\r\nefgh\r\nijkl", Separator::Newline)]
    #[case::null("abcd\0efgh\0ijkl", Separator::Null)]
    fn read_lines(#[case] input: &str, #[case] separator: Separator) {
        let mut reader = LineReader::new(B(input), separator, 8);
        assert_ok_eq!(reader.read_line(), Some(B("abcd")));
        assert_ok_eq!(reader.read_line(), Some(B("efgh")));
        assert_ok_eq!(reader.read_line(), Some(B("ijkl")));
        assert_ok_eq!(reader.read_line(), None);
    }

    #[test]
    fn read_lines_err() {
        let mut reader = LineReader::new(B("abcdefgh"), Separator::Newline, 8);
        assert_err!(reader.read_line());
    }
}
