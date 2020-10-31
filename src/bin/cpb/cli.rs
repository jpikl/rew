use clap::{AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run::Options;
use common::transfer::TransferOptions;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
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
/// Each pair of source and destination path must be either both files or both directories. Mixing these types will result in error.
///
/// Source path must exist. Using non-existent source path will result in error.
///
/// Destination path may exist. Existing destination file will be overwritten. Existing destination directory will have its contents merged with contents of source directory.
///
/// Missing parent directories in destination path will be created as needed.
pub struct Cli {
    /// Read instructions delimited by NUL, not newline
    #[clap(short = 'z', long)]
    pub read_nul: bool,

    /// Continue after an error, fail at end
    #[clap(short = 's', long)]
    pub fail_at_end: bool,

    /// Explain what is being done
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

impl Options for Cli {
    fn color(&self) -> Option<ColorChoice> {
        self.color
    }
}

impl TransferOptions for Cli {
    fn read_nul(&self) -> bool {
        self.read_nul
    }

    fn verbose(&self) -> bool {
        self.verbose
    }

    fn fail_at_end(&self) -> bool {
        self.fail_at_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        assert!(Cli::try_parse_from(&["cpb"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["cpb", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }

    #[test]
    fn read_nul() {
        let cli = Cli::try_parse_from(&["cpb", "--read-nul"]).unwrap();
        assert_eq!(TransferOptions::read_nul(&cli), true);
    }

    #[test]
    fn verbose() {
        let cli = Cli::try_parse_from(&["cpb", "--verbose"]).unwrap();
        assert_eq!(TransferOptions::verbose(&cli), true);
    }

    #[test]
    fn fail_at_end() {
        let cli = Cli::try_parse_from(&["cpb", "--fail-at-end"]).unwrap();
        assert_eq!(TransferOptions::fail_at_end(&cli), true);
    }
}
