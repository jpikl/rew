use crate::counter;
use clap::{crate_name, crate_version, AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run::Options;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    after_help = "Use `-h` for short descriptions and `--help` for more details.",
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    setting(AppSettings::UnifiedHelpMessage),
    setting(AppSettings::DontCollapseArgsInUsage),
)]
/// Rewrite FS paths according to a pattern
pub struct Cli {
    /// Output pattern
    ///
    /// If not provided, input values are directly written to stdout.
    ///
    /// Use `--explain` flag to print explanation of a given pattern.
    /// Use `--help-pattern` flag to print description of patter syntax.
    /// Use `--help-filters` flag to print filter reference.
    #[clap(verbatim_doc_comment)]
    pub pattern: Option<String>,

    /// Input values (read from stdin by default)
    #[clap(value_name = "value")]
    pub values: Vec<String>,

    /// Read values delimited by a specific character, not newline
    #[clap(
    short = 'd',
    long,
    value_name = "char",
    conflicts_with_all = &["read-nul", "read-raw"],
    parse(try_from_str = parse_single_byte_char)
    )]
    pub read: Option<u8>,

    /// Read values delimited by NUL, not newline
    #[clap(short = 'z', long, conflicts_with_all = &["read-raw", "read"])]
    pub read_nul: bool,

    /// Read the whole input into memory as a single value
    #[clap(short = 'r', long, conflicts_with_all = &["read-nul", "read"])]
    pub read_raw: bool,

    /// Print results delimited by a specific string, not newline
    #[clap(
    short = 'D',
    long,
    value_name = "string",
    conflicts_with_all = &["print-nul", "print-raw"]
    )]
    pub print: Option<String>,

    /// Print results delimited by NUL, not newline
    #[clap(short = 'Z', long, conflicts_with_all = &["print-raw", "print"])]
    pub print_nul: bool,

    /// Print results without a delimiter
    #[clap(short = 'R', long, conflicts_with_all = &["print-nul", "print"])]
    pub print_raw: bool,

    /// Enable diff output mode
    ///
    /// Respects with other `--print*` flags.
    /// Prints machine-readable transformations as results:
    ///
    ///    <input_value_1
    ///    >output_value_1
    ///    <input_value_2
    ///    >output_value_2
    ///    ...
    ///    <input_value_N
    ///    >output_value_N
    ///
    /// Such output can be processed by accompanying `mvb` and `cpb` utilities to perform bulk move/copy.
    #[clap(short = 'b', long, conflicts_with = "pretty", verbatim_doc_comment)]
    pub diff: bool,

    /// Enable pretty output mode
    ///
    /// Ignores other `--print*` flags.
    /// Prints human-readable transformations as results:
    ///
    ///     input_value_1 -> output_value_1
    ///     input_value_2 -> output_value_2
    ///     ...
    ///     input_value_N -> output_value_N
    #[clap(short = 'p', long, conflicts_with = "diff", verbatim_doc_comment)]
    pub pretty: bool,

    /// When to use colors
    #[clap(
        long,
        value_name = "when",
        possible_values = COLOR_VALUES,
        parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,

    /// Continue processing after an error, fail at end
    #[clap(short = 'e', long)]
    pub fail_at_end: bool,

    /// Local counter configuration
    ///
    /// init - Initial value.
    /// step - Value increment (default: 1)
    #[clap(short = 'c', long, value_name = "init[:step]", verbatim_doc_comment)]
    pub local_counter: Option<counter::Config>,

    /// Global counter configuration
    ///
    /// init - Initial value.
    /// step - Value increment (default: 1).
    #[clap(short = 'C', long, value_name = "init[:step]", verbatim_doc_comment)]
    pub global_counter: Option<counter::Config>,

    /// Custom escape character to use in pattern
    #[clap(long, value_name = "char")]
    pub escape: Option<char>,

    /// Print explanation of a given pattern
    #[clap(long, requires = "pattern")]
    pub explain: bool,

    /// Print help information
    #[clap(short = 'h', long)]
    pub help: bool,

    /// Print description of pattern syntax
    #[clap(long)]
    pub help_pattern: bool,

    /// Print filter reference
    #[clap(long)]
    pub help_filters: bool,

    /// Print version information
    #[clap(long)]
    pub version: bool,
}

impl Options for Cli {
    fn color(&self) -> Option<ColorChoice> {
        self.color
    }
}

pub fn parse_single_byte_char(string: &str) -> Result<u8, &'static str> {
    if string.chars().count() != 1 {
        Err("value must be a single character")
    } else if string.len() != 1 {
        Err("multi-byte characters are not supported")
    } else {
        Ok(string.as_bytes()[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        assert!(Cli::try_parse_from(&["rew", "pattern"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["rew", "pattern", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }

    #[test]
    fn parses_single_byte_char() {
        assert_eq!(parse_single_byte_char("a"), Ok(b'a'));
        assert_eq!(
            parse_single_byte_char("á"),
            Err("multi-byte characters are not supported",)
        );
        assert_eq!(
            parse_single_byte_char("aa"),
            Err("value must be a single character")
        );
    }
}
