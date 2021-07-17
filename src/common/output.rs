use std::error::Error;
use std::io::{Result, Write};

use termcolor::{Color, WriteColor};

use crate::color::spec_color;

pub fn write_error<O: Write + WriteColor, E: Error>(output: &mut O, error: &E) -> Result<()> {
    output.set_color(&spec_color(Color::Red))?;
    write!(output, "error:")?;
    output.reset()?;
    writeln!(output, " {}", error)
}

#[cfg(test)]
pub mod tests {
    use std::io::{self, ErrorKind};

    use super::*;
    use crate::testing::{ColoredOuput, OutputChunk};

    #[test]
    fn write_error() {
        let mut output = ColoredOuput::new();
        let error = io::Error::new(ErrorKind::InvalidData, "message");
        super::write_error(&mut output, &error).unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Red, "error:"),
                OutputChunk::plain(" message\n")
            ]
        );
    }
}
