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
    use crate::output_test::{ColoredBuffer, ColoredChunk};
    use std::io::{self, ErrorKind};

    #[test]
    fn write_error_ok() {
        let mut buffer = ColoredBuffer::new();
        write_error(
            &mut buffer,
            &io::Error::new(ErrorKind::InvalidData, "message"),
        )
        .unwrap();
        assert_eq!(
            buffer.chunks(),
            vec![
                ColoredChunk::color(Color::Red, "error:"),
                ColoredChunk::plain(" message\n")
            ]
        );
    }
}
