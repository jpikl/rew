use crate::color::{spec_bold_color, spec_color};
use std::fmt;
use std::io::{Result, Write};
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(PartialEq)]
pub struct ColoredChunk {
    pub spec: ColorSpec,
    pub value: String,
}

impl ColoredChunk {
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

impl fmt::Debug for ColoredChunk {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "ColoredChunk::")?;

        match (self.spec.fg(), self.spec.bold()) {
            (None, _) => write!(fmt, "plain(")?,
            (Some(color), true) => write!(fmt, "bold_color(Color::{:?}, ", color)?,
            (Some(color), false) => write!(fmt, "color(Color::{:?}, ", color)?,
        }

        write!(fmt, "{:?})", self.value.replace("\n", "\\n"))
    }
}

#[derive(Default)]
pub struct ColoredBuffer {
    spec: ColorSpec,
    chunks: Vec<ColoredChunk>,
}

impl ColoredBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chunks(self) -> Vec<ColoredChunk> {
        self.chunks
    }
}

impl Write for ColoredBuffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let spec = &self.spec;
        let value = std::str::from_utf8(buf).unwrap();

        if let Some(chunk) = self.chunks.last_mut().filter(|chunk| &chunk.spec == spec) {
            chunk.value += value;
        } else {
            self.chunks.push(ColoredChunk {
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

impl WriteColor for ColoredBuffer {
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
