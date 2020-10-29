use clap::{AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run;
use regex::Regex;
use std::path::PathBuf;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    verbatim_doc_comment
)]
/// Rewrite FS paths using a pattern.
///
/// Accompanying utilities `mvb` and `cpb` can be used to move/copy files based on `rew` output:
///
///   $ find -name '*.txt' | rew -b '{p}.bak' | cpb
pub struct Cli {
    /// Output pattern
    pub pattern: String,

    /// Paths to rewrite (read from stdin by default)
    #[clap(value_name = "path")]
    pub paths: Vec<PathBuf>,

    /// Read paths delimited by NUL, not newline
    #[clap(short = 'z', long, conflicts_with = "read-raw")]
    pub read_nul: bool,

    /// Read the whole input as a single path
    #[clap(short = 'r', long, conflicts_with = "read-nul")]
    pub read_raw: bool,

    /// Print results delimited by NUL, not newline
    #[clap(short = 'Z', long, conflicts_with = "print-raw")]
    pub print_nul: bool,

    /// Print results without any delimiter
    #[clap(short = 'R', long, conflicts_with = "print-nul")]
    pub print_raw: bool,

    /// Print machine-readable transformations as a results
    #[clap(short = 'b', long, conflicts_with = "pretty")]
    pub bulk: bool,

    /// Print human-readable transformations as a results
    #[clap(short = 'p', long, conflicts_with = "bulk")]
    pub pretty: bool,

    /// Continue after a path processing error, fail at end
    #[clap(short = 's', long)]
    pub fail_at_end: bool,

    /// Print explanation of a given pattern
    #[clap(long)]
    pub explain: bool,

    /// Regular expression matched against file name
    #[clap(short = 'e', long)]
    pub regex: Option<Regex>,

    /// Regular expression matched against path
    #[clap(short = 'E', long, value_name = "regex")]
    pub regex_full: Option<Regex>,

    /// Global counter initial value
    #[clap(long, value_name = "number")]
    pub gc_init: Option<u32>,

    /// Global counter step
    #[clap(long, value_name = "number")]
    pub gc_step: Option<u32>,

    /// Local counter initial value
    #[clap(long, value_name = "number")]
    pub lc_init: Option<u32>,

    /// Local counter step
    #[clap(long, value_name = "number")]
    pub lc_step: Option<u32>,

    /// Custom escape character to use in pattern
    #[clap(long, value_name = "char")]
    pub escape: Option<char>,

    /// When to use colors
    #[clap(
        long,
        value_name = "when",
        possible_values = COLOR_VALUES,
        parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,
}

impl run::Cli for Cli {
    fn color(&self) -> Option<ColorChoice> {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        assert!(Cli::try_parse_from(&["cmd", "pattern"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["cmd", "pattern", "--color=always"]).unwrap();
        assert_eq!(run::Cli::color(&cli), Some(ColorChoice::Always));
    }
}
