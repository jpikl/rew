use std::io::{Result, Write};
use std::path::Path;

use termcolor::{Color, WriteColor};

use crate::color::spec_color;
use crate::transfer::fs::TransferMode;

pub struct TransferLog<O: Write + WriteColor> {
    output: O,
}

impl<O: Write + WriteColor> TransferLog<O> {
    pub fn new(output: O) -> Self {
        Self { output }
    }

    pub fn begin_transfer(
        &mut self,
        mode: TransferMode,
        src_path: &Path,
        dst_path: &Path,
    ) -> Result<()> {
        let action = match mode {
            TransferMode::Move => "Moving",
            TransferMode::Copy => "Copying",
        };
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
        self.end_transfer(Color::Green, "OK")
    }

    pub fn end_with_failure(&mut self) -> Result<()> {
        self.end_transfer(Color::Red, "FAILED")
    }

    pub fn end_transfer(&mut self, color: Color, result: &str) -> Result<()> {
        self.output.set_color(&spec_color(color))?;
        write!(self.output, "{}", result)?;
        self.output.reset()?;
        writeln!(self.output)
    }
}

#[cfg(test)]
pub mod tests {
    use test_case::test_case;

    use super::*;
    use crate::testing::{ColoredOuput, OutputChunk};

    #[test_case(TransferMode::Move, "Moving"  ; "move ")]
    #[test_case(TransferMode::Copy, "Copying" ; "copy")]
    fn begin_transfer(mode: TransferMode, output_action: &str) {
        let mut output = ColoredOuput::new();

        TransferLog::new(&mut output)
            .begin_transfer(mode, &Path::new("a/b.c"), &Path::new("d/e.f"))
            .unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::plain(&format!("{} '", output_action)),
                OutputChunk::color(Color::Blue, "a/b.c"),
                OutputChunk::plain("' to '"),
                OutputChunk::color(Color::Blue, "d/e.f"),
                OutputChunk::plain("' ... ")
            ]
        );
    }

    #[test]
    fn end_with_success() {
        let mut output = ColoredOuput::new();
        TransferLog::new(&mut output).end_with_success().unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Green, "OK"),
                OutputChunk::plain("\n")
            ]
        );
    }

    #[test]
    fn end_with_failure() {
        let mut output = ColoredOuput::new();
        TransferLog::new(&mut output).end_with_failure().unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Red, "FAILED"),
                OutputChunk::plain("\n")
            ]
        );
    }
}
