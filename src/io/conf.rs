use bstr::ByteSlice;

pub trait LineConfig {
    fn line_separator(&self) -> LineSeparator;
}

pub trait BufSizeConfig {
    fn buf_size(&self) -> usize;
}

pub trait BufModeConfig {
    fn buf_full(&self) -> bool;
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
