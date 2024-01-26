use super::conf::LineConfig;
use super::conf::LineSeparator;
use super::conf::OPTIMAL_IO_BUF_SIZE;
use anyhow::Result;
use std::io::copy;
use std::io::stdout;
use std::io::BufWriter;
use std::io::Read;
use std::io::StdoutLock;
use std::io::Write;

pub trait WriterConfig {
    fn write_is_buffered(&self) -> bool;
}

pub struct Writer<W: Write> {
    inner: BufWriter<W>,
    separator: u8,
    buffered: bool,
}

impl Writer<StdoutLock<'static>> {
    pub fn from_stdout(config: &(impl LineConfig + WriterConfig)) -> Self {
        Self::new(
            stdout().lock(),
            config.line_separator(),
            config.write_is_buffered(),
        )
    }
}

impl<W: Write> Writer<W> {
    pub fn new(inner: W, separator: LineSeparator, buffered: bool) -> Self {
        Self {
            inner: BufWriter::with_capacity(OPTIMAL_IO_BUF_SIZE, inner),
            separator: separator.as_byte(),
            buffered,
        }
    }

    pub fn write_line(&mut self, line: &[u8]) -> Result<()> {
        if self.buffered {
            self.inner.write_all(line)?;
            self.inner.write_all(&[self.separator])?;
        } else {
            self.inner.get_mut().write_all(line)?;
            self.inner.get_mut().write_all(&[self.separator])?;
        }
        Ok(())
    }

    pub fn write_block(&mut self, block: &[u8]) -> Result<()> {
        if self.buffered {
            self.inner.write_all(block)?;
        } else {
            self.inner.get_mut().write_all(block)?;
        }
        Ok(())
    }

    pub fn write_all_from(&mut self, source: &mut impl Source) -> Result<()> {
        if self.buffered {
            self.inner.write_all(source.empty_buf())?;
            copy(source.reader(), &mut self.inner)?;
        } else {
            self.inner.get_mut().write_all(source.empty_buf())?;
            copy(source.reader(), &mut self.inner.get_mut())?;
        }
        Ok(())
    }
}

pub trait Source {
    type Reader: Read;

    fn empty_buf(&mut self) -> &[u8] {
        &[]
    }

    fn reader(&mut self) -> &mut Self::Reader;
}

impl<R: Read> Source for R {
    type Reader = R;

    fn reader(&mut self) -> &mut Self::Reader {
        self
    }
}
