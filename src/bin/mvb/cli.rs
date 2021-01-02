use clap::{crate_version, AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::help::highlight_static;
use common::run::Options;
use common::transfer::TransferOptions;
use indoc::indoc;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
    name = "mvb",
    version = crate_version!(),
    long_about = highlight_static(indoc!{"
        Bulk move (rename) files and directories

        `mvb` reads instructions from standard input in the following format:
       
            <src_path_1
            >dst_path_1
            <src_path_2
            >dst_path_2
            ...
            <src_path_N
            >dst_path_N
       
        Such input can be generated using accompanying `rew` utility and its `-b, --diff` flag:
       
            $> find -name '*.jpeg' | rew -b '{B}.jpg' | mvb # Rename all *.jpeg files to *.jpg
       
        Each pair of source and destination path must be either both files or both directories. Mixing these types will result in error.
       
        Source path must exist. Using non-existent source path will result in error.
       
        Destination path may exist. Existing destination file will be overwritten. Existing destination directory will have its contents merged with contents of source directory.
       
        Missing parent directories in destination path will be created as needed.
       
        Nothing will be done if source and destination paths point to the same file or directory.
    "}),
    after_help = highlight_static("Use `-h` for short descriptions and `--help` for more details."),
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    setting(AppSettings::DontCollapseArgsInUsage),
    setting(AppSettings::UnifiedHelpMessage),
)]
/// Bulk move (rename) files and directories
pub struct Cli {
    /// Read instructions delimited by NUL, not newline
    #[clap(short = 'z', long)]
    pub read_nul: bool,

    /// Continue processing after an error, fail at end
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
    use claim::*;
    use ntest::*;

    #[test]
    fn default() {
        assert_ok!(Cli::try_parse_from(&["mvb"]));
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["mvb", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }

    #[test]
    fn read_nul() {
        let cli = Cli::try_parse_from(&["mvb", "--read-nul"]).unwrap();
        assert_true!(TransferOptions::read_nul(&cli));
    }

    #[test]
    fn verbose() {
        let cli = Cli::try_parse_from(&["mvb", "--verbose"]).unwrap();
        assert_true!(TransferOptions::verbose(&cli));
    }

    #[test]
    fn fail_at_end() {
        let cli = Cli::try_parse_from(&["mvb", "--fail-at-end"]).unwrap();
        assert_true!(TransferOptions::fail_at_end(&cli));
    }
}
