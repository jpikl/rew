use crate::utils::ByteSize;
use bstr::ByteSlice;
use bstr::decode_last_utf8;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::io::copy;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    BufferFull(usize),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BufferFull(len) => {
                write!(f, "Unable to fit input data into buffer ({})", ByteSize(*len))
            }
            Self::Io(err) => err.fmt(f),
        }
    }
}

pub trait Chunker {
    // Returns (chunk_length, bytes_consumed)
    fn find_chunk(slice: &[u8]) -> (usize, usize);
}

pub struct LineChunker;
pub struct RecordChunker;
pub struct CharChunker;

impl Chunker for LineChunker {
    fn find_chunk(slice: &[u8]) -> (usize, usize) {
        match slice.find_byte(b'\n') {
            Some(pos) => {
                if pos > 0 && slice[pos - 1] == b'\r' {
                    (pos - 1, pos + 1)
                } else {
                    (pos, pos + 1)
                }
            }
            None => (0, 0),
        }
    }
}

impl Chunker for RecordChunker {
    fn find_chunk(slice: &[u8]) -> (usize, usize) {
        match slice.find_byte(b'\0') {
            Some(pos) => (pos, pos + 1),
            None => (0, 0),
        }
    }
}

impl Chunker for CharChunker {
    fn find_chunk(slice: &[u8]) -> (usize, usize) {
        let len = match decode_last_utf8(slice) {
            (None, remainder) => slice.len() - remainder,
            _ => slice.len(),
        };
        (len, len)
    }
}

pub struct ChunkReader<R, C> {
    inner: R,
    buf: Vec<u8>,
    start: usize, // Start offset of unprocessed buf area
    end: usize,   // End offset of unprocessed buf area
    _chunker: PhantomData<C>,
}

impl<R: Read, C: Chunker> ChunkReader<R, C> {
    pub fn new(inner: R, buf: Vec<u8>) -> Self {
        Self {
            inner,
            buf,
            start: 0,
            end: 0,
            _chunker: PhantomData,
        }
    }

    pub fn read_chunk(&mut self) -> Result<Option<&mut [u8]>, Error> {
        loop {
            // Find the next chunk in unprocessed buffer area.
            let (len, consumed) = C::find_chunk(&self.buf[self.start..self.end]);
            if consumed > 0 {
                let chunk = &mut self.buf[self.start..][..len];
                self.start += consumed;
                return Ok(Some(chunk));
            }

            // Compact data in the buffer to get more free space at its end.
            if self.start > 0 {
                self.buf.copy_within(self.start..self.end, 0);
                self.end -= self.start;
                self.start = 0;
            }

            // Try to read more data into the buffer to complete a chunk.
            let len = match self.inner.read(&mut self.buf[self.end..]) {
                Ok(len) => len,
                Err(err) => return Err(Error::Io(err)),
            };

            if len > 0 {
                self.end += len;
                continue; // Re-try chunk detection.
            }

            // End of input (the buffer is empty).
            if self.end == 0 {
                return Ok(None);
            }

            // End of input (the buffer contains unterminated chunk).
            if self.end < self.buf.len() {
                let remainder = &mut self.buf[self.start..self.end];
                self.start = self.end;
                return Ok(Some(remainder));
            }

            // The whole chunk could not fit into the buffer.
            return Err(Error::BufferFull(self.buf.len()));
        }
    }
}

pub struct ByteChunkReader<R> {
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> ByteChunkReader<R> {
    pub fn new(inner: R, buf: Vec<u8>) -> Self {
        Self { inner, buf }
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn read_chunk(&mut self) -> std::io::Result<Option<&mut [u8]>> {
        match self.inner.read(&mut self.buf) {
            Ok(0) => Ok(None),
            Ok(len) => Ok(Some(&mut self.buf[..len])),
            Err(err) => Err(err),
        }
    }
}

pub type CharChunkReader<R> = ChunkReader<R, CharChunker>;

pub enum LineReader<R> {
    Lines(ChunkReader<R, LineChunker>),
    Records(ChunkReader<R, RecordChunker>),
}

impl<R: Read> LineReader<R> {
    pub fn lines(inner: R, buf: Vec<u8>) -> Self {
        Self::Lines(ChunkReader::new(inner, buf))
    }

    pub fn records(inner: R, buf: Vec<u8>) -> Self {
        Self::Records(ChunkReader::new(inner, buf))
    }

    pub fn read_line(&mut self) -> Result<Option<&mut [u8]>, Error> {
        match self {
            Self::Lines(reader) => reader.read_chunk(),
            Self::Records(reader) => reader.read_chunk(),
        }
    }
}

pub struct Writer<W: Write> {
    inner: WriterInner<W>,
    separator: u8,
}

enum WriterInner<W: Write> {
    Buffered(BufWriter<W>),
    Unbuffered(W),
}

impl<W: Write> Writer<W> {
    pub fn buffered(inner: W, separator: u8, buf_size: usize) -> Self {
        Self {
            inner: WriterInner::Buffered(BufWriter::with_capacity(buf_size, inner)),
            separator,
        }
    }

    pub fn unbuffered(inner: W, separator: u8) -> Self {
        Self {
            inner: WriterInner::Unbuffered(inner),
            separator,
        }
    }

    #[cfg(test)]
    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            WriterInner::Buffered(inner) => inner.flush(),
            WriterInner::Unbuffered(inner) => inner.flush(),
        }
    }

    #[cfg(test)]
    fn get_ref(&self) -> &W {
        match &self.inner {
            WriterInner::Buffered(inner) => inner.get_ref(),
            WriterInner::Unbuffered(inner) => inner,
        }
    }

    pub fn write_line(&mut self, line: &[u8]) -> std::io::Result<()> {
        self.write(line)?;
        self.write_separator()
    }

    pub fn write_separator(&mut self) -> std::io::Result<()> {
        self.write(&[self.separator])
    }

    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self.inner {
            WriterInner::Buffered(ref mut inner) => inner.write_all(buf),
            WriterInner::Unbuffered(ref mut inner) => inner.write_all(buf),
        }
    }

    pub fn write_all_from(&mut self, reader: &mut impl Read) -> std::io::Result<u64> {
        match self.inner {
            WriterInner::Buffered(ref mut inner) => copy(reader, inner),
            WriterInner::Unbuffered(ref mut inner) => copy(reader, inner),
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
    #[case("", 0, 0)]
    #[case("a", 0, 0)]
    #[case("\n", 0, 1)]
    #[case("\r\n", 0, 2)]
    #[case("a\n", 1, 2)]
    #[case("a\nb", 1, 2)]
    #[case("a\n\n", 1, 2)]
    #[case("a\r\n", 1, 3)]
    #[case("a\r\nb", 1, 3)]
    #[case("a\r\n\n", 1, 3)]
    fn line_chunker(#[case] input: &str, #[case] len: usize, #[case] consumed: usize) {
        assert_eq!(LineChunker::find_chunk(B(input)), (len, consumed));
    }

    #[rstest]
    #[case("", 0, 0)]
    #[case("a", 0, 0)]
    #[case("\0", 0, 1)]
    #[case("a\0", 1, 2)]
    #[case("a\0b", 1, 2)]
    #[case("a\0\0", 1, 2)]
    fn record_chunker(#[case] input: &str, #[case] len: usize, #[case] consumed: usize) {
        assert_eq!(RecordChunker::find_chunk(B(input)), (len, consumed));
    }

    // \xf0\x92\x80\x80 represents sumerian character 𒀀
    #[rstest]
    #[case(b"", 0, 0)]
    #[case(b"\xf0", 0, 0)]
    #[case(b"\xf0\x92", 0, 0)]
    #[case(b"\xf0\x92\x80", 0, 0)]
    #[case(b"\xf0\x92\x80\x80", 4, 4)]
    #[case(b"a", 1, 1)]
    #[case(b"a\xf0", 1, 1)]
    #[case(b"a\xf0\x92", 1, 1)]
    #[case(b"a\xf0\x92\x80", 1, 1)]
    #[case(b"a\xf0\x92\x80\x80", 5, 5)]
    fn char_chunker(#[case] input: &[u8], #[case] len: usize, #[case] consumed: usize) {
        assert_eq!(CharChunker::find_chunk(input), (len, consumed));
    }

    // ByteChunkReader should completely ignore UTF-8 character bundaries.
    // The test data mimics exactly the unit test for CharChunkReader.
    #[rstest]
    #[case(b"", &[])]
    // Valid byte sequence + Incomplete 3 byte character
    #[case(b"abcd\xf0\x92\x80", &[B(b"abcd\xf0\x92\x80")])]
    #[case(b"abcde\xf0\x92\x80", &[B(b"abcde\xf0\x92\x80")])]
    #[case(b"abcdef\xf0\x92\x80", &[B(b"abcdef\xf0\x92"), B(b"\x80")])]
    // Valid byte sequence + Incomplete 3 byte character + Valid byte
    #[case(b"abc\xf0\x92\x80e", &[B(b"abc\xf0\x92\x80e")])]
    #[case(b"abcd\xf0\x92\x80e", &[B(b"abcd\xf0\x92\x80e")])]
    #[case(b"abcde\xf0\x92\x80e", &[B(b"abcde\xf0\x92\x80"), B("e")])]
    // Valid byte sequence + Complete 4 byte character
    #[case(b"abc\xf0\x92\x80\x80", &[B(b"abc\xf0\x92\x80\x80")])]
    #[case(b"abcd\xf0\x92\x80\x80", &[B(b"abcd\xf0\x92\x80\x80")])]
    #[case(b"abcde\xf0\x92\x80\x80", &[B(b"abcde\xf0\x92\x80"), B(b"\x80")])]
    // Invalid byte sequence + Incomplete 3 byte character
    #[case(b"\x80\x80\x80\x80\xf0\x92\x80", &[B(b"\x80\x80\x80\x80\xf0\x92\x80")])]
    #[case(b"\x80\x80\x80\x80\x80\xf0\x92\x80", &[B(b"\x80\x80\x80\x80\x80\xf0\x92\x80")])]
    #[case(b"\x80\x80\x80\x80\x80\x80\xf0\x92\x80", &[B(b"\x80\x80\x80\x80\x80\x80\xf0\x92"), B(b"\x80")])]
    // Invalid byte sequence + Complete 4 byte character
    #[case(b"\x80\x80\x80\xf0\x92\x80\x80", &[B(b"\x80\x80\x80\xf0\x92\x80\x80")])]
    #[case(b"\x80\x80\x80\x80\xf0\x92\x80\x80", &[B(b"\x80\x80\x80\x80\xf0\x92\x80\x80")])]
    #[case(b"\x80\x80\x80\x80\x80\xf0\x92\x80\x80", &[B(b"\x80\x80\x80\x80\x80\xf0\x92\x80"), B(b"\x80")])]
    fn byte_chunk_reader(#[case] input: &[u8], #[case] output: &[&[u8]]) {
        let mut reader = ByteChunkReader::new(input, vec![0; 8]);
        for output_item in output {
            assert_some_eq!(assert_ok!(reader.read_chunk()), B(output_item));
        }
        assert_ok_eq!(reader.read_chunk(), None);
    }

    // CharChunkReader should respect UTF-8 character boundaries
    // It should never split input in the middle of a valid UTF-8 character.
    #[rstest]
    #[case(b"", &[])]
    // Valid byte sequence + Incomplete 3 byte character
    #[case(b"abcd\xf0\x92\x80", &[B(b"abcd"), B(b"\xf0\x92\x80")])]
    #[case(b"abcde\xf0\x92\x80", &[B(b"abcde"), B(b"\xf0\x92\x80")])]
    #[case(b"abcdef\xf0\x92\x80", &[B(b"abcdef"), B(b"\xf0\x92\x80")])]
    // Valid byte sequence + Incomplete 3 byte character + Valid byte
    #[case(b"abc\xf0\x92\x80e", &[B(b"abc\xf0\x92\x80e")])]
    #[case(b"abcd\xf0\x92\x80e", &[B(b"abcd\xf0\x92\x80e")])]
    #[case(b"abcde\xf0\x92\x80e", &[B(b"abcde"), B(b"\xf0\x92\x80e")])]
    // Valid byte sequence + Complete 4 byte character
    #[case(b"abc\xf0\x92\x80\x80", &[B(b"abc\xf0\x92\x80\x80")])]
    #[case(b"abcd\xf0\x92\x80\x80", &[B(b"abcd\xf0\x92\x80\x80")])]
    #[case(b"abcde\xf0\x92\x80\x80", &[B(b"abcde"), B(b"\xf0\x92\x80\x80")])] // Must not split the 4 byte character!
    // Invalid byte sequence + Incomplete 3 byte character
    #[case(b"\x80\x80\x80\x80\xf0\x92\x80", &[B(b"\x80\x80\x80\x80"), B(b"\xf0\x92\x80")])]
    #[case(b"\x80\x80\x80\x80\x80\xf0\x92\x80", &[B(b"\x80\x80\x80\x80\x80"), B(b"\xf0\x92\x80")])]
    #[case(b"\x80\x80\x80\x80\x80\x80\xf0\x92\x80", &[B(b"\x80\x80\x80\x80\x80\x80"), B(b"\xf0\x92\x80")])]
    // Invalid byte sequence + Complete 4 byte character
    #[case(b"\x80\x80\x80\xf0\x92\x80\x80", &[B(b"\x80\x80\x80\xf0\x92\x80\x80")])]
    #[case(b"\x80\x80\x80\x80\xf0\x92\x80\x80", &[B(b"\x80\x80\x80\x80\xf0\x92\x80\x80")])]
    #[case(b"\x80\x80\x80\x80\x80\xf0\x92\x80\x80", &[B(b"\x80\x80\x80\x80\x80"), B(b"\xf0\x92\x80\x80")])] // Must not split the 4 byte character!
    fn char_chunk_reader(#[case] input: &[u8], #[case] output: &[&[u8]]) {
        let mut reader = CharChunkReader::new(input, vec![0; 8]);
        for output_item in output {
            assert_some_eq!(assert_ok!(reader.read_chunk()), B(output_item));
        }
        assert_ok_eq!(reader.read_chunk(), None);
    }

    #[test]
    fn char_chunk_reader_err() {
        let mut reader = CharChunkReader::new(B(b"\xf0\x92\x80\x80"), vec![0; 3]);
        let err = assert_err!(reader.read_chunk());
        assert_eq!(err.to_string(), "Unable to fit input data into buffer (3 B)");
    }

    #[rstest]
    #[case("", &[], LineReader::lines)]
    #[case("", &[], LineReader::records)]
    #[case("\n\n", &["", ""], LineReader::lines)]
    #[case("\n\r\n", &["", ""], LineReader::lines)]
    #[case("\r\n\n", &["", ""], LineReader::lines)]
    #[case("\r\n\r\n", &["", ""], LineReader::lines)]
    #[case("\0\0", &["", ""], LineReader::records)]
    #[case("abcd\nefgh\nijkl", &["abcd", "efgh", "ijkl"], LineReader::lines)]
    #[case("abcd\nefgh\nijkl\n", &["abcd", "efgh", "ijkl"], LineReader::lines)]
    #[case("abcd\r\nefgh\r\nijkl\r\n", &["abcd", "efgh", "ijkl"], LineReader::lines)]
    #[case("abcd\0efgh\0ijkl", &["abcd", "efgh", "ijkl"], LineReader::records)]
    #[case("abcd\0efgh\0ijkl\0", &["abcd", "efgh", "ijkl"], LineReader::records)]
    fn line_reader<'a>(
        #[case] input: &'a str,
        #[case] output: &'a [&'a str],
        #[case] construct: fn(&'a [u8], Vec<u8>) -> LineReader<&'a [u8]>,
    ) {
        let mut reader = construct(B(input), vec![0; 8]);
        for output_item in output {
            assert_some_eq!(assert_ok!(reader.read_line()), B(output_item));
        }
        assert_ok_eq!(reader.read_line(), None);
    }

    #[rstest]
    #[case("abcdefgh", LineReader::lines)]
    #[case("abcdefgh\n", LineReader::lines)]
    #[case("abcdefg\r\n", LineReader::lines)]
    #[case("abcdefgh", LineReader::records)]
    #[case("abcdefgh\0", LineReader::records)]
    fn read_lines_err<'a>(#[case] input: &'a str, #[case] construct: fn(&'a [u8], Vec<u8>) -> LineReader<&'a [u8]>) {
        let mut reader = construct(B(input), vec![0; 8]);
        let err = assert_err!(reader.read_line());
        assert_eq!(err.to_string(), "Unable to fit input data into buffer (8 B)");
    }

    #[rstest]
    #[case(Writer::buffered(vec![], b'\n', 8))]
    #[case(Writer::unbuffered(vec![], b'\n'))]
    fn write(#[case] mut writer: Writer<Vec<u8>>) {
        assert_ok!(writer.write_line(B("abcd")));
        assert_ok!(writer.write_line(B("efgh")));
        assert_ok!(writer.write(B("ijkl")));
        assert_ok!(writer.write_separator());
        assert_ok!(writer.write_all_from(&mut B("mnop")));
        assert_ok!(writer.flush());
        assert_eq!(writer.get_ref(), B("abcd\nefgh\nijkl\nmnop"));
    }
}
