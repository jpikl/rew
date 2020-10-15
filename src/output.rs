use crate::color::spec_color;
use crate::io::Output;
use std::error::Error;
use std::io::Result;
use termcolor::Color;

pub fn write_error<O: Output, E: Error>(output: &mut O, error: &E) -> Result<()> {
    output.set_color(&spec_color(Color::Red))?;
    write!(output, "error:")?;
    output.reset()?;
    writeln!(output, " {}", error)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::io::mem::{MemoryOutput, OutputChunk};
    use std::io::{self, ErrorKind};

    #[test]
    fn write_error_ok() {
        let mut output = MemoryOutput::new();
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
}
