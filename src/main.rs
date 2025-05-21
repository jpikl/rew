mod cli;
mod commands;
mod global;
mod io;
mod run;
mod utils;

use cli::Command;
use cli::CommandBuilder;
use commands::cat::CAT;
use commands::first::FIRST;
use commands::lower::LOWER;
use commands::prefix::PREFIX;
use commands::quote::QUOTE;
use commands::skip::SKIP;
use commands::suffix::SUFFIX;
use commands::upper::UPPER;
use global::BUF_MODE;
use global::BUF_SIZE;
use global::HELP;
use global::NULL;
use global::VERSION;

const REW: Command = CommandBuilder::new()
    .name(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .version(env!("CARGO_PKG_VERSION"))
    .options(&[HELP.arg, VERSION.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .subcommands(&[CAT, PREFIX, SUFFIX, QUOTE, LOWER, UPPER, FIRST, SKIP])
    .done();

fn main() {
    REW.parse_args().run();
}
