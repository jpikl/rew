use clap::{crate_version, AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run::Options;
use common::transfer::TransferOptions;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
    name = "mvb",
    version = crate_version!(),
    after_help = "Use `-h` for short descriptions and `--help` for more details.",
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    setting(AppSettings::UnifiedHelpMessage),
    setting(AppSettings::DontCollapseArgsInUsage),
    verbatim_doc_comment,
)]
/// Bulk move (rename) files and directories
///
/// `mvb` reads instructions from standard input in the following format:
///
///     <src_path_1
///     >dst_path_1
///     <src_path_2
///     >dst_path_2
///     ...
///     <src_path_N
///     >dst_path_N
///
/// Such input can be generated using accompanying `rew` utility and its `-b, --diff` flag:
///
///     $> find -name '*.jpeg' | rew -b '{N6-}.jpg' | mvb # Rename all *.jpeg files to *.jpg
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

    /// Continue processing after an error, fail at end
    #[clap(short = 'e', long)]
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

    /// Print help information
    #[clap(short = 'h', long)]
    pub help: bool,

    /// Print version information
    #[clap(long)]
    pub version: bool,
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
        assert!(Cli::try_parse_from(&["mvb"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["mvb", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }

    #[test]
    fn read_nul() {
        let cli = Cli::try_parse_from(&["mvb", "--read-nul"]).unwrap();
        assert_eq!(TransferOptions::read_nul(&cli), true);
    }

    #[test]
    fn verbose() {
        let cli = Cli::try_parse_from(&["mvb", "--verbose"]).unwrap();
        assert_eq!(TransferOptions::verbose(&cli), true);
    }

    #[test]
    fn fail_at_end() {
        let cli = Cli::try_parse_from(&["mvb", "--fail-at-end"]).unwrap();
        assert_eq!(TransferOptions::fail_at_end(&cli), true);
    }
}
