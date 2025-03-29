use memchr::memchr;
use std::fmt::Display;
use std::io::Read;

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

fn trim_newline(line: &[u8]) -> &[u8] {
    if let [without_lf @ .., b'\n'] = line {
        if let [without_crlf @ .., b'\r'] = without_lf {
            return without_crlf;
        }
        return without_lf;
    }
    line
}

fn trim_null(line: &[u8]) -> &[u8] {
    if let [without_null @ .., b'\0'] = line {
        return without_null;
    }
    line
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    BufferFull(usize),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BufferFull(len) => {
                write!(f, "unable to fit input line into buffer ({len}B)")
            }
            Self::Io(err) => err.fmt(f),
        }
    }
}

pub struct LineReader<R> {
    inner: R,
    separator: u8,
    trim: fn(&[u8]) -> &[u8],
    buf: Vec<u8>,
    start: usize, // Start of unprocessed buf area
    end: usize,   // End of unprocessed buf area
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

    pub fn read_line(&mut self) -> Result<Option<&[u8]>, Error> {
        loop {
            // Find the next line ending in unprocessed buffer area
            if let Some(pos) = memchr(self.separator, &self.buf[self.start..self.end]) {
                let line = &self.buf[self.start..][..=pos];
                self.start += pos + 1;
                return Ok(Some((self.trim)(line)));
            }

            // Compact data in the buffer to get more free space at its end
            if self.start > 0 {
                self.buf.copy_within(self.start..self.end, 0);
                self.end -= self.start;
                self.start = 0;
            }

            // Try to fetch more data into the buffer to complete a line
            let len = match self.inner.read(&mut self.buf[self.end..]) {
                Ok(len) => len,
                Err(err) => return Err(Error::Io(err)),
            };

            if len > 0 {
                self.end += len;
                continue; // Re-try line ending detection
            }

            // End of input (the buffer is empty)
            if self.end == 0 {
                return Ok(None);
            }

            // End of input (the buffer contains unterminated line)
            if self.end < self.buf.len() {
                let remainder = &self.buf[self.start..self.end];
                self.start = self.end;
                return Ok(Some(remainder));
            }

            // Input line could not fit into the buffer
            return Err(Error::BufferFull(self.buf.len()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bstr::B;
    use claims::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "")]
    #[case("\n", "")]
    #[case("\r\n", "")]
    #[case("a\n", "a")]
    #[case("a\r\n", "a")]
    fn trim_newline(#[case] input: &str, #[case] output: &str) {
        assert_eq!(super::trim_newline(B(input)), B(output));
    }

    #[rstest]
    #[case("", "")]
    #[case("\0", "")]
    #[case("a\0", "a")]
    fn trim_null(#[case] input: &str, #[case] output: &str) {
        assert_eq!(super::trim_null(B(input)), B(output));
    }

    #[rstest]
    #[case("abcd\nefgh\nijkl", Separator::Newline)]
    #[case("abcd\r\nefgh\r\nijkl", Separator::Newline)]
    #[case("abcd\0efgh\0ijkl", Separator::Null)]
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
        let err = assert_err!(reader.read_line());
        assert_eq!(err.to_string(), "unable to fit input line into buffer (8B)");
    }
}
