use common::color::{parse_color, COLOR_VALUES};
use common::run;
use structopt::{clap::AppSettings, StructOpt};
use termcolor::ColorChoice;

#[derive(Debug, StructOpt)]
#[structopt(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    about = "Bulk move (rename) files and directories"
)]
pub struct Cli {
    /// Reads items delimited by NUL, not newline
    #[structopt(short = "z", long)]
    pub read_nul: bool,

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
