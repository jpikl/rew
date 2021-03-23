use clap::{crate_version, AppSettings, Clap};
use common::color::{parse_color, COLOR_CHOICES};
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
    /// Read instructions terminated by NUL character, not newline
    #[clap(short = 'z', long)]
    pub read_nul: bool,

    /// Continue processing after an error, fail at end
    #[clap(short = 'F', long)]
    pub fail_at_end: bool,

    /// Explain what is being done
    #[clap(short = 'v', long)]
    pub verbose: bool,

    /// When to use colors
    #[clap(
        long,
        value_name = "when",
        possible_values = COLOR_CHOICES,
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
    use test_case::test_case;

    #[test_case(&[], None; "default")]
    #[test_case(&["--color=always"], Some(ColorChoice::Always); "always")]
    fn color(args: &[&str], result: Option<ColorChoice>) {
        assert_eq!(run(args).color(), result);
    }

    #[test_case(&[], false; "off")]
    #[test_case(&["--read-nul"], true; "on")]
    fn read_nul(args: &[&str], result: bool) {
        assert_eq!(run(args).read_nul(), result);
    }

    #[test_case(&[], false; "off")]
    #[test_case(&["--verbose"], true; "on")]
    fn verbose(args: &[&str], result: bool) {
        assert_eq!(run(args).verbose(), result);
    }

    #[test_case(&[], false; "off")]
    #[test_case(&["--fail-at-end"], true; "on")]
    fn fail_at_end(args: &[&str], result: bool) {
        assert_eq!(run(args).fail_at_end(), result);
    }

    fn run(args: &[&str]) -> Cli {
        Cli::try_parse_from(&[&["mvb"], args].concat()).unwrap()
    }
}
