// See https://misc.flogisoft.com/bash/tip_colors_and_formatting

use clap::builder::StyledStr;
use std::io;
use std::io::Write;

pub const RESET: &str = "\x1b[0m";

pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";

pub const BOLD: &str = "\x1b[1m";
pub const BOLD_RED: &str = "\x1b[1;31m";

pub fn write_help(writer: &mut impl Write, help: &StyledStr) -> io::Result<()> {
    let mut is_code = false;

    // Colorize text between `...` quotes as bold
    for part in help.ansi().to_string().split('`') {
        if is_code {
            write!(writer, "{BOLD}{part}{RESET}")?;
        } else {
            writer.write_all(part.as_bytes())?;
        }

        is_code = !is_code;
    }

    Ok(())
}
