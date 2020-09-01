use std::error::Error;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, WriteColor};

pub mod input;

pub const COLOR_VALUES: &[&str] = &[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI];

const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";

pub fn parse_color(string: &str) -> Result<ColorChoice, &'static str> {
    match string {
        COLOR_ALWAYS => Ok(ColorChoice::Always),
        COLOR_ANSI => Ok(ColorChoice::AlwaysAnsi),
        COLOR_AUTO => Ok(ColorChoice::Auto),
        COLOR_NEVER => Ok(ColorChoice::Never),
        _ => Err("invalid value"),
    }
}

pub fn detect_color(color: Option<ColorChoice>) -> ColorChoice {
    match color {
        Some(ColorChoice::Auto) | None => {
            if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        }
        Some(other) => other,
    }
}

pub fn spec_color(color: Color) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color));
    spec
}

pub fn spec_bold_color(color: Color) -> ColorSpec {
    let mut spec = spec_color(color);
    spec.set_bold(true);
    spec
}

pub fn write_error<S: Write + WriteColor, E: Error>(stream: &mut S, error: &E) -> io::Result<()> {
    stream.set_color(&spec_color(Color::Red))?;
    write!(stream, "error:")?;
    stream.reset()?;
    writeln!(stream, " {}", error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_ok() {
        assert_eq!(parse_color(COLOR_ALWAYS), Ok(ColorChoice::Always));
        assert_eq!(parse_color(COLOR_ANSI), Ok(ColorChoice::AlwaysAnsi));
        assert_eq!(parse_color(COLOR_AUTO), Ok(ColorChoice::Auto));
        assert_eq!(parse_color(COLOR_NEVER), Ok(ColorChoice::Never));
    }

    #[test]
    fn parse_color_err() {
        assert_eq!(parse_color(""), Err("invalid value"));
        assert_eq!(parse_color("x"), Err("invalid value"));
    }
}
