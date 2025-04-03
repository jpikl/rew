use crate::cli::Args;
use crate::global::BUF_SIZE;
use crate::global::ByteSize;
use crate::global::NULL;
use crate::io::LineReader;
use crate::io::Separator;
use std::io::StdinLock;

pub struct Context<'a> {
    args: &'a Args,
}

impl<'a> Context<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self { args }
    }

    pub fn line_reader(&self) -> LineReader<StdinLock> {
        let ByteSize(buf_size) = self.args.get(&BUF_SIZE);

        let separator = if self.args.get(&NULL) {
            Separator::Null
        } else {
            Separator::Newline
        };

        LineReader::new(std::io::stdin().lock(), separator, buf_size)
    }
}
