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
