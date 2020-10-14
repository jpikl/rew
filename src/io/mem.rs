use crate::color::{spec_bold_color, spec_color};
use crate::io::Io;
use std::fmt;
use std::io::{BufRead, Read, Result, Write};
use std::sync::{Arc, Mutex, MutexGuard};
use termcolor::{Color, ColorSpec, WriteColor};

pub struct MemoryIo {
    stdin: Arc<Mutex<MemoryInput>>,
    stdout: Arc<Mutex<MemoryOutput>>,
    stderr: Arc<Mutex<MemoryOutput>>,
}

impl MemoryIo {
    pub fn new(data: &'static [u8]) -> Self {
        Self {
            stdin: Arc::new(Mutex::new(MemoryInput::new(data))),
            stdout: Arc::new(Mutex::new(MemoryOutput::new())),
            stderr: Arc::new(Mutex::new(MemoryOutput::new())),
        }
    }

    pub fn stdout_chunks(&self) -> Vec<OutputChunk> {
        self.stdout
            .lock()
            .expect("Unable to lock memory stdout")
            .chunks()
            .clone()
    }

    pub fn stderr_chunks(&self) -> Vec<OutputChunk> {
        self.stderr
            .lock()
            .expect("Unable to lock memory stderr")
            .chunks()
            .clone()
    }
}

impl<'a> Io<'a> for MemoryIo {
    type StdinLock = MemoryInputLock<'a>;
    type StdoutLock = MemoryOutputLock<'a>;
    type StderrLock = MemoryOutputLock<'a>;

    fn stdin(&'a self) -> Self::StdinLock {
        MemoryInputLock(self.stdin.lock().expect("Unable to lock memory stdin"))
    }

    fn stdout(&'a self) -> Self::StdoutLock {
        MemoryOutputLock(self.stdout.lock().expect("Unable to lock memory stdout"))
    }

    fn stderr(&'a self) -> Self::StderrLock {
        MemoryOutputLock(self.stderr.lock().expect("Unable to lock memory stderr"))
    }
}

pub struct MemoryInputLock<'a>(MutexGuard<'a, MemoryInput>);
pub struct MemoryOutputLock<'a>(MutexGuard<'a, MemoryOutput>);

pub struct MemoryInput {
    data: &'static [u8],
}

impl MemoryInput {
    pub fn new(data: &'static [u8]) -> Self {
        Self { data }
    }
}

impl Read for MemoryInput {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.data.read(buf)
    }
}

impl<'a> Read for MemoryInputLock<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}

impl BufRead for MemoryInput {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.data.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.data.consume(amt)
    }
}

impl<'a> BufRead for MemoryInputLock<'a> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.0.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.0.consume(amt)
    }
}

#[derive(Default)]
pub struct MemoryOutput {
    spec: ColorSpec,
    chunks: Vec<OutputChunk>,
}

impl MemoryOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chunks(&self) -> &Vec<OutputChunk> {
        &self.chunks
    }
}

impl Write for MemoryOutput {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let spec = &self.spec;
        let value = std::str::from_utf8(buf).unwrap();

        if let Some(chunk) = self.chunks.last_mut().filter(|chunk| &chunk.spec == spec) {
            chunk.value += value;
        } else {
            self.chunks.push(OutputChunk {
                spec: self.spec.clone(),
                value: String::from(value),
            })
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<'a> Write for MemoryOutputLock<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush()
    }
}

impl WriteColor for MemoryOutput {
    fn supports_color(&self) -> bool {
        true
    }

    fn set_color(&mut self, spec: &ColorSpec) -> Result<()> {
        self.spec = spec.clone();
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.spec = ColorSpec::new();
        Ok(())
    }
}

impl<'a> WriteColor for MemoryOutputLock<'a> {
    fn supports_color(&self) -> bool {
        self.0.supports_color()
    }

    fn set_color(&mut self, spec: &ColorSpec) -> Result<()> {
        self.0.set_color(spec)
    }

    fn reset(&mut self) -> Result<()> {
        self.0.reset()
    }
}

#[derive(PartialEq, Clone)]
pub struct OutputChunk {
    pub spec: ColorSpec,
    pub value: String,
}

impl OutputChunk {
    pub fn plain(value: &str) -> Self {
        Self {
            spec: ColorSpec::new(),
            value: String::from(value),
        }
    }

    pub fn color(color: Color, value: &str) -> Self {
        Self {
            spec: spec_color(color),
            value: String::from(value),
        }
    }

    pub fn bold_color(color: Color, value: &str) -> Self {
        Self {
            spec: spec_bold_color(color),
            value: String::from(value),
        }
    }
}

impl fmt::Debug for OutputChunk {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "OutputChunk::")?;

        match (self.spec.fg(), self.spec.bold()) {
            (None, _) => write!(fmt, "plain(")?,
            (Some(color), true) => write!(fmt, "bold_color(Color::{:?}, ", color)?,
            (Some(color), false) => write!(fmt, "color(Color::{:?}, ", color)?,
        }

        write!(fmt, "{:?})", self.value.replace("\n", "\\n"))
    }
}
