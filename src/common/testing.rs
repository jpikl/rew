use crate::color::{spec_bold_color, spec_color};
use std::fmt;
use std::io::{Error, ErrorKind, Result, Write};
use termcolor::{Color, ColorSpec, WriteColor};

pub fn unpack_io_error(error: Error) -> (ErrorKind, String) {
    (error.kind(), error.to_string())
}

#[derive(Default)]
pub struct ColoredOuput {
    spec: ColorSpec,
    chunks: Vec<OutputChunk>,
}

impl ColoredOuput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chunks(&self) -> &Vec<OutputChunk> {
        &self.chunks
    }
}

impl Write for ColoredOuput {
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

impl WriteColor for ColoredOuput {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::*;

    #[test]
    fn unpack_io_error() {
        use super::*;

        assert_eq!(
            unpack_io_error(Error::new(ErrorKind::Other, "test")),
            (ErrorKind::Other, String::from("test"))
        );
    }

    mod colored_output {
        use super::*;

        #[test]
        fn supports_color() {
            assert_true!(ColoredOuput::new().supports_color());
        }

        #[test]
        fn write() {
            let mut output = ColoredOuput::new();

            write!(output, "a").unwrap();
            write!(output, "b").unwrap();
            output.set_color(&spec_color(Color::Red)).unwrap();
            write!(output, "c").unwrap();
            write!(output, "d").unwrap();
            output.set_color(&spec_bold_color(Color::Blue)).unwrap();
            write!(output, "e").unwrap();
            write!(output, "f").unwrap();
            output.reset().unwrap();
            write!(output, "g").unwrap();
            output.flush().unwrap();

            assert_eq!(
                output.chunks,
                &[
                    OutputChunk::plain("ab"),
                    OutputChunk::color(Color::Red, "cd"),
                    OutputChunk::bold_color(Color::Blue, "ef"),
                    OutputChunk::plain("g"),
                ]
            );
        }
    }

    mod output_chunk {
        use super::*;

        mod init {
            use super::*;

            #[test]
            fn plain() {
                let chunk = OutputChunk::plain("ab");
                assert_eq!(chunk.spec, ColorSpec::new());
                assert_eq!(chunk.value, "ab");
            }

            #[test]
            fn color() {
                let chunk = OutputChunk::color(Color::Red, "cd");
                assert_eq!(chunk.spec, spec_color(Color::Red));
                assert_eq!(chunk.value, "cd");
            }

            #[test]
            fn bold_color() {
                let chunk = OutputChunk::bold_color(Color::Blue, "ef");
                assert_eq!(chunk.spec, spec_bold_color(Color::Blue));
                assert_eq!(chunk.value, "ef");
            }
        }

        mod display {
            use super::*;

            #[test]
            fn plain() {
                assert_eq!(
                    format!("{:?}", OutputChunk::plain("a\nb")),
                    r#"OutputChunk::plain("a\\nb")"#
                );
            }

            #[test]
            fn color() {
                assert_eq!(
                    format!("{:?}", OutputChunk::color(Color::Red, "c\nd")),
                    r#"OutputChunk::color(Color::Red, "c\\nd")"#
                );
            }

            #[test]
            fn bold_color() {
                assert_eq!(
                    format!("{:?}", OutputChunk::bold_color(Color::Blue, "e\nf")),
                    r#"OutputChunk::bold_color(Color::Blue, "e\\nf")"#
                );
            }
        }
    }
}
