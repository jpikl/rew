use bstr::ByteSlice;

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library https://github.com/Freaky/rust-linereader.
pub const OPTIMAL_IO_BUF_SIZE: usize = 32 * 1024;

pub trait LineConfig {
    fn line_separator(&self) -> LineSeparator;
}

#[derive(Copy, Clone)]
pub enum LineSeparator {
    Newline,
    Null,
}

impl LineSeparator {
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
