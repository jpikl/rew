use common::color::{parse_color, COLOR_VALUES};
use common::run;
use structopt::{clap::AppSettings, StructOpt};
use termcolor::ColorChoice;

#[derive(Debug, StructOpt)]
#[structopt(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    verbatim_doc_comment
)]
/// Bulk copy files and directories.
///
/// `cpb` reads instructions from standard input in the following format:
///
/// <src_path_1
/// >dst_path_1
/// <src_path_2
/// >dst_path_2
/// ...
/// <src_path_N
/// >dst_path_N
///
/// Such input can be generated using accompanying `rew` utility and its `-b, --bulk` flag:
///
///   $ find -name '*.txt' | rew -b '{p}.bak' | cpb
///
/// By default:
/// - Attempt to overwrite an existing file will result in error unless `-f, --force` or `n, --no-clobber` flag is used.
/// - Attempt to copy a file to a non-existent directory will result in error unless `-p, --parents` flag is used.
/// - Attempt to copy a non-empty directory will result in error unless `-r, --recursive` flag is used.
pub struct Cli {
    /// Reads instructions delimited by NUL, not newline
    #[structopt(short = "z", long)]
    pub read_nul: bool,

    /// Overrides existing files
    #[structopt(short = "f", long, conflicts_with = "no-clobber")]
    pub force: bool,

    /// Does not override existing files
    #[structopt(short = "n", long, conflicts_with = "force")]
    pub no_clobber: bool,

    /// Makes parent directories as needed
    #[structopt(short = "p", long)]
    pub parents: bool,

    /// Copies directories recursively
    #[structopt(short = "r", long)]
    pub recursive: bool,

    /// Continues after an error, fails at end
    #[structopt(short = "s", long)]
    pub fail_at_end: bool,

    /// Explains what is being done
    #[structopt(short = "v", long)]
    pub verbose: bool,

    /// When to use colors
    #[structopt(
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
        assert!(Cli::from_iter_safe(&["cmd"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::from_iter_safe(&["cmd", "--color=always"]).unwrap();
        assert_eq!(run::Cli::color(&cli), Some(ColorChoice::Always));
    }
}
