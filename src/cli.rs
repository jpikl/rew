use regex::Regex;
use std::path::PathBuf;
use structopt::{clap::AppSettings, StructOpt};
use termcolor::ColorChoice;

const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";

#[derive(Debug, StructOpt)]
#[structopt(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub struct Cli {
    /// Output pattern
    pub pattern: String,

    /// Paths to process (read from stdin by default)
    #[structopt(value_name = "path")]
    pub paths: Vec<PathBuf>,

    /// Reads paths delimited by NUL, not newline
    #[structopt(short = "z", long, conflicts_with = "read-raw")]
    pub read_nul: bool,

    /// Reads the whole input as a single path
    #[structopt(short = "r", long, conflicts_with = "read-nul")]
    pub read_raw: bool,

    /// Prints results delimited by NUL, not newline
    #[structopt(short = "Z", long, conflicts_with = "print-raw")]
    pub print_nul: bool,

    /// Prints results without any delimiter
    #[structopt(short = "R", long, conflicts_with = "print-null")]
    pub print_raw: bool,

    /// Prints machine-readable transformations as a results
    #[structopt(short = "b", long, conflicts_with = "pretty")]
    pub batch: bool,

    /// Prints human-readable transformations as a results
    #[structopt(short = "p", long, conflicts_with = "batch")]
    pub pretty: bool,

    /// Continues after a path processing error, fails at end
    #[structopt(short = "s", long)]
    pub fail_at_end: bool,

    /// Prints explanation of a given pattern
    #[structopt(long)]
    pub explain: bool,

    /// Regular expression matched against file name
    #[structopt(short = "e", long)]
    pub regex: Option<Regex>,

    /// Regular expression matched against path
    #[structopt(short = "E", long, value_name = "regex")]
    pub regex_full: Option<Regex>,

    /// Global counter initial value
    #[structopt(long, value_name = "number")]
    pub gc_init: Option<u32>,

    /// Global counter step
    #[structopt(long, value_name = "number")]
    pub gc_step: Option<u32>,

    /// Local counter initial value
    #[structopt(long, value_name = "number")]
    pub lc_init: Option<u32>,

    /// Local counter step
    #[structopt(long, value_name = "number")]
    pub lc_step: Option<u32>,

    /// Custom escape character to use in pattern
    #[structopt(long, value_name = "char")]
    pub escape: Option<char>,

    /// When to use colors
    #[structopt(
        long,
        value_name = "when",
        possible_values = &[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI],
        parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,
}

fn parse_color(string: &str) -> Result<ColorChoice, &'static str> {
    match string {
        COLOR_ALWAYS => Ok(ColorChoice::Always),
        COLOR_ANSI => Ok(ColorChoice::AlwaysAnsi),
        COLOR_AUTO => Ok(ColorChoice::Auto),
        COLOR_NEVER => Ok(ColorChoice::Never),
        _ => Err("invalid value"),
    }
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
