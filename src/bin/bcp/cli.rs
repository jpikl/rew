use common::color::{parse_color, COLOR_VALUES};
use structopt::{clap::AppSettings, StructOpt};
use termcolor::ColorChoice;

#[derive(Debug, StructOpt)]
#[structopt(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    about = "Batch copy files and directories"
)]
pub struct Cli {
    /// When to use colors
    #[structopt(
        long,
        value_name = "when",
        possible_values = COLOR_VALUES,
        parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,
}
