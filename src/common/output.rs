use crate::color::spec_color;
use std::error::Error;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

pub fn write_error<O: Write + WriteColor, E: Error>(output: &mut O, error: &E) -> Result<()> {
    output.set_color(&spec_color(Color::Red))?;
    write!(output, "error:")?;
    output.reset()?;
    writeln!(output, " {}", error)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::testing::{ColoredOuput, OutputChunk};
    use std::io::{self, ErrorKind};

    #[test]
    fn write_error_ok() {
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
}
