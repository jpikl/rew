use crate::color::{spec_bold_color, spec_color};
use crate::utils::str_from_utf8;
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
        let value = str_from_utf8(buf)?;

        if let Some(chunk) = self.chunks.last_mut().filter(|chunk| &chunk.spec == spec) {
            chunk.value += value;
        } else {
            self.chunks.push(OutputChunk {
                spec: self.spec.clone(),
                value: value.into(),
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
            value: value.into(),
        }
    }

    pub fn color(color: Color, value: &str) -> Self {
        Self {
            spec: spec_color(color),
            value: value.into(),
        }
    }

    pub fn bold_color(color: Color, value: &str) -> Self {
        Self {
            spec: spec_bold_color(color),
            value: value.into(),
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
        assert_eq!(
            super::unpack_io_error(Error::new(ErrorKind::Other, "test")),
            (ErrorKind::Other, "test".into())
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
        use test_case::test_case;

        #[test_case(OutputChunk::plain("ab"),                   ColorSpec::new(),             "ab" ; "plain")]
        #[test_case(OutputChunk::color(Color::Red, "cd"),       spec_color(Color::Red),       "cd" ; "color")]
        #[test_case(OutputChunk::bold_color(Color::Blue, "ef"), spec_bold_color(Color::Blue), "ef" ; "bold color")]
        fn create(chunk: OutputChunk, spec: ColorSpec, value: &str) {
            assert_eq!(chunk.spec, spec);
            assert_eq!(chunk.value, value);
        }

        #[test_case(OutputChunk::plain("a\nb"),                   r#"OutputChunk::plain("a\\nb")"#                   ; "plain")]
        #[test_case(OutputChunk::color(Color::Red, "c\nd"),       r#"OutputChunk::color(Color::Red, "c\\nd")"#       ; "color")]
        #[test_case(OutputChunk::bold_color(Color::Blue, "e\nf"), r#"OutputChunk::bold_color(Color::Blue, "e\\nf")"# ; "bold color")]
        fn debug(chunk: OutputChunk, result: &str) {
            assert_eq!(format!("{:?}", chunk), result);
        }
    }
}
