use crate::color::spec_color;
use std::error::Error;
use std::io::{Result, Write};
use std::path::Path;
use termcolor::{Color, WriteColor};

pub fn write_error<O: Write + WriteColor, E: Error>(output: &mut O, error: &E) -> Result<()> {
    output.set_color(&spec_color(Color::Red))?;
    write!(output, "error:")?;
    output.reset()?;
    writeln!(output, " {}", error)
}

pub struct Log<O: Write + WriteColor> {
    output: O,
}

impl<O: Write + WriteColor> Log<O> {
    pub fn new(output: O) -> Self {
        Self { output }
    }

    pub fn begin_move(&mut self, src_path: &Path, dst_path: &Path) -> Result<()> {
        self.begin_action("Moving", src_path, dst_path)
    }

    pub fn begin_copy(&mut self, src_path: &Path, dst_path: &Path) -> Result<()> {
        self.begin_action("Copying", src_path, dst_path)
    }

    fn begin_action(&mut self, action: &str, src_path: &Path, dst_path: &Path) -> Result<()> {
        write!(self.output, "{} '", action)?;
        self.output.set_color(&spec_color(Color::Blue))?;
        write!(self.output, "{}", src_path.to_string_lossy())?;
        self.output.reset()?;
        write!(self.output, "' to '")?;
        self.output.set_color(&spec_color(Color::Blue))?;
        write!(self.output, "{}", dst_path.to_string_lossy())?;
        self.output.reset()?;
        write!(self.output, "' ... ")?;
        self.output.flush()
    }

    pub fn end_with_success(&mut self) -> Result<()> {
        self.end_action(Color::Green, "OK")
    }

    pub fn end_with_failure(&mut self) -> Result<()> {
        self.end_action(Color::Red, "FAILED")
    }

    pub fn end_action(&mut self, color: Color, result: &str) -> Result<()> {
        self.output.set_color(&spec_color(color))?;
        write!(self.output, "{}", result)?;
        self.output.reset()?;
        writeln!(self.output)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::testing::{ColoredOuput, OutputChunk};
    use std::io::{self, ErrorKind};

    #[test]
    fn writes_error() {
        let mut output = ColoredOuput::new();
        write_error(
            &mut output,
            &io::Error::new(ErrorKind::InvalidData, "message"),
        )
        .unwrap();
        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Red, "error:"),
                OutputChunk::plain(" message\n")
            ]
        );
    }

    #[test]
    fn log_begins_move() {
        let mut output = ColoredOuput::new();
        Log::new(&mut output)
            .begin_move(&Path::new("a/b.c"), &Path::new("d/e.f"))
            .unwrap();
        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::plain("Moving '"),
                OutputChunk::color(Color::Blue, "a/b.c"),
                OutputChunk::plain("' to '"),
                OutputChunk::color(Color::Blue, "d/e.f"),
                OutputChunk::plain("' ... ")
            ]
        );
    }

    #[test]
    fn log_begins_copy() {
        let mut output = ColoredOuput::new();
        Log::new(&mut output)
            .begin_copy(&Path::new("a/b.c"), &Path::new("d/e.f"))
            .unwrap();
        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::plain("Copying '"),
                OutputChunk::color(Color::Blue, "a/b.c"),
                OutputChunk::plain("' to '"),
                OutputChunk::color(Color::Blue, "d/e.f"),
                OutputChunk::plain("' ... ")
            ]
        );
    }

    #[test]
    fn log_ends_with_success() {
        let mut output = ColoredOuput::new();
        Log::new(&mut output).end_with_success().unwrap();
        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Green, "OK"),
                OutputChunk::plain("\n")
            ]
        );
    }

    #[test]
    fn log_ends_with_failure() {
        let mut output = ColoredOuput::new();
        Log::new(&mut output).end_with_failure().unwrap();
        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Red, "FAILED"),
                OutputChunk::plain("\n")
            ]
        );
    }
}
