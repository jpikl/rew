use crate::color::spec_color;
use std::error::Error;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

pub fn write_error<S: Write + WriteColor, E: Error>(stream: &mut S, error: &E) -> Result<()> {
    stream.set_color(&spec_color(Color::Red))?;
    write!(stream, "error:")?;
    stream.reset()?;
    writeln!(stream, " {}", error)
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
