// See https://misc.flogisoft.com/bash/tip_colors_and_formatting

use std::io;
use std::io::Write;
use std::str;

pub const RESET: &str = "\x1b[0m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";

pub const BOLD: &str = "\x1b[1m";
pub const BOLD_RED: &str = "\x1b[1;31m";

pub struct Colorizer {
    pub quote_char: char,
    pub quote_color: &'static str,
}

impl Colorizer {
    pub fn write(&self, writer: &mut impl Write, input: impl AsRef<str>) -> io::Result<()> {
        let mut inside = false;

        for part in input.as_ref().split(self.quote_char) {
            if inside {
                write!(writer, "{}{part}{RESET}", self.quote_color)?;
            } else {
                writer.write_all(part.as_bytes())?;
            }
            inside = !inside;
        }

        Ok(())
    }

    pub fn to_string(&self, input: impl AsRef<str>) -> io::Result<String> {
        let mut buffer = Vec::new();
        self.write(&mut buffer, input)?;
        Ok(String::from_utf8(buffer).expect("could not convert colorized text to utf-8"))
    }
}
