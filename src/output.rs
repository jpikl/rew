use crate::color::spec_color;
use std::error::Error;
use std::io::{self, Write};
use termcolor::{Color, WriteColor};

pub fn write_error<S: Write + WriteColor, E: Error>(stream: &mut S, error: &E) -> io::Result<()> {
    stream.set_color(&spec_color(Color::Red))?;
    write!(stream, "error:")?;
    stream.reset()?;
    writeln!(stream, " {}", error)
}
