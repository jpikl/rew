use termcolor::ColorChoice;

use clap::{AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run;

#[derive(Debug, Clap)]
#[clap(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    verbatim_doc_comment
)]
/// Bulk move (rename) files and directories.
///
/// `mvb` reads instructions from standard input in the following format:
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
///   $ find -name '*.txt' | rew -b '{p}.bak' | mvb
///
/// Each pair of source and destination path must be either both files or both directories. Mixing these types will result in error.
///
/// Source path must always exist, destination path may not. Using non-existent source path will result in error.
///
/// Attempt to overwrite an existing file will result in error unless `-f, --force` or `n, --no-clobber` flag is used.
///
/// Attempt to use a non-existent parent directory in destination path will result in error unless `-p, --parents` flag is used.
///
/// Attempt to move a non-empty directory between different filesystems will result in error unless `-r, --recursive` flag is used.
///
/// Attempt to merge two different non-empty directories will result in error unless `-r, --recursive` flag is used.
pub struct Cli {
    /// Reads items delimited by NUL, not newline
    #[clap(short = 'z', long)]
    pub read_nul: bool,

    /// Overrides existing files
    #[clap(short = 'f', long, conflicts_with = "no-clobber")]
    pub force: bool,

    /// Does not override existing files
    #[clap(short = 'n', long, conflicts_with = "force")]
    pub no_clobber: bool,

    /// Makes parent directories as needed
    #[clap(short = 'p', long)]
    pub parents: bool,

    /// Move directories recursively between different filesystems
    #[clap(short = 'r', long)]
    pub recursive: bool,

    /// Continues after an error, fails at end
    #[clap(short = 's', long)]
    pub fail_at_end: bool,

    /// Explains what is being done
    #[clap(short = 'v', long)]
    pub verbose: bool,

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
        assert!(Cli::try_parse_from(&["cmd"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["cmd", "--color=always"]).unwrap();
        assert_eq!(run::Cli::color(&cli), Some(ColorChoice::Always));
    }
}
