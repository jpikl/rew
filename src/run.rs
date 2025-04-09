use crate::cli::Args;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::BufMode;
use crate::global::NULL;
use crate::io::ByteChunkReader;
use crate::io::CharChunkReader;
use crate::io::LineReader;
use crate::io::Writer;
use std::io::StdinLock;
use std::io::StdoutLock;

pub struct Context<'a> {
    args: &'a Args,
}

impl<'a> Context<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self { args }
    }

    #[allow(clippy::unused_self)]
    pub fn raw_reader(&self) -> StdinLock<'_> {
        std::io::stdin().lock()
    }

    pub fn byte_chunk_reader(&self) -> ByteChunkReader<StdinLock<'_>> {
        ByteChunkReader::new(self.raw_reader(), self.zeroed_buf())
    }

    pub fn char_chunk_reader(&self) -> CharChunkReader<StdinLock<'_>> {
        CharChunkReader::new(self.raw_reader(), self.zeroed_buf())
    }

    pub fn line_reader(&self) -> LineReader<StdinLock<'_>> {
        match self.args.get(&NULL) {
            true => LineReader::records(self.raw_reader(), self.zeroed_buf()),
            false => LineReader::lines(self.raw_reader(), self.zeroed_buf()),
        }
    }

    #[allow(clippy::unused_self)]
    pub fn raw_writer(&self) -> StdoutLock<'_> {
        std::io::stdout().lock()
    }

    pub fn writer(&self) -> Writer<StdoutLock<'_>> {
        match self.buf_mode() {
            BufMode::Full => Writer::buffered(self.raw_writer(), self.separator(), self.buf_size()),
            BufMode::Line => Writer::unbuffered(self.raw_writer(), self.separator()),
        }
    }

    pub fn zeroed_buf(&self) -> Vec<u8> {
        vec![0u8; self.buf_size()]
    }

    pub fn uninit_buf(&self) -> Vec<u8> {
        Vec::with_capacity(self.buf_size())
    }

    pub fn buf_size(&self) -> usize {
        self.args.get(&BUF_SIZE).0
    }

    pub fn buf_mode(&self) -> BufMode {
        self.args.get(&BUF_MODE)
    }

    pub fn separator(&self) -> u8 {
        match self.args.get(&NULL) {
            true => b'\0',
            false => b'\n',
        }
    }
}
