use crate::cli::Context;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::BufMode;
use crate::global::NULL;
use crate::global::PATH_STYLE;
use crate::global::PathStyle;
use crate::io::ByteChunkReader;
use crate::io::CharChunkReader;
use crate::io::LineReader;
use crate::io::Writer;
use std::io::StdinLock;
use std::io::StdoutLock;

pub trait ContextExt {
    fn raw_reader(&self) -> StdinLock<'_>;
    fn byte_chunk_reader(&self) -> ByteChunkReader<StdinLock<'_>>;
    fn char_chunk_reader(&self) -> CharChunkReader<StdinLock<'_>>;
    fn line_reader(&self) -> LineReader<StdinLock<'_>>;
    fn raw_writer(&self) -> StdoutLock<'_>;
    fn writer(&self) -> Writer<StdoutLock<'_>>;
    fn zeroed_buf(&self) -> Vec<u8>;
    fn uninit_buf(&self) -> Vec<u8>;
    fn buf_size(&self) -> usize;
    fn buf_mode(&self) -> BufMode;
    fn separator(&self) -> u8;
    fn path_style(&self) -> PathStyle;
}

impl<'a> ContextExt for Context<'a> {
    fn raw_reader(&self) -> StdinLock<'_> {
        std::io::stdin().lock()
    }

    fn byte_chunk_reader(&self) -> ByteChunkReader<StdinLock<'_>> {
        ByteChunkReader::new(self.raw_reader(), self.zeroed_buf())
    }

    fn char_chunk_reader(&self) -> CharChunkReader<StdinLock<'_>> {
        CharChunkReader::new(self.raw_reader(), self.zeroed_buf())
    }

    fn line_reader(&self) -> LineReader<StdinLock<'_>> {
        match self.args.get(&NULL) {
            true => LineReader::records(self.raw_reader(), self.zeroed_buf()),
            false => LineReader::lines(self.raw_reader(), self.zeroed_buf()),
        }
    }

    fn raw_writer(&self) -> StdoutLock<'_> {
        std::io::stdout().lock()
    }

    fn writer(&self) -> Writer<StdoutLock<'_>> {
        match self.buf_mode() {
            BufMode::Full => Writer::buffered(self.raw_writer(), self.separator(), self.buf_size()),
            BufMode::Line => Writer::unbuffered(self.raw_writer(), self.separator()),
        }
    }

    fn zeroed_buf(&self) -> Vec<u8> {
        vec![0u8; self.buf_size()]
    }

    fn uninit_buf(&self) -> Vec<u8> {
        Vec::with_capacity(self.buf_size())
    }

    fn buf_size(&self) -> usize {
        self.args.get(&BUF_SIZE).0
    }

    fn buf_mode(&self) -> BufMode {
        self.args.get(&BUF_MODE)
    }

    fn separator(&self) -> u8 {
        match self.args.get(&NULL) {
            true => b'\0',
            false => b'\n',
        }
    }

    fn path_style(&self) -> PathStyle {
        self.args.get(&PATH_STYLE)
    }
}
