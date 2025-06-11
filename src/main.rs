mod cli;
mod commands;
mod format;
mod global;
mod io;
mod run;
mod utils;

use crate::commands::ascii::ASCII;
use cli::Command;
use cli::CommandBuilder;
use commands::base::BASE;
use commands::cat::CAT;
use commands::ext::EXT;
use commands::first::FIRST;
use commands::r#loop::LOOP;
use commands::lower::LOWER;
use commands::prefix::PREFIX;
use commands::quote::QUOTE;
use commands::rand::RAND;
use commands::seq::SEQ;
use commands::skip::SKIP;
use commands::stream::STREAM;
use commands::suffix::SUFFIX;
use commands::trim::TRIM;
use commands::upper::UPPER;
use commands::uuid::UUID;
use global::BUF_MODE;
use global::BUF_SIZE;
use global::HELP;
use global::HELP_CMD;
use global::NULL;
use global::VERSION;

const REW: Command = CommandBuilder::new()
    .name(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .version(env!("CARGO_PKG_VERSION"))
    .options(&[&HELP.arg, &VERSION.arg, &NULL.arg, &BUF_SIZE.arg, &BUF_MODE.arg])
    .subcommands(&[
        &BASE, &EXT, // Path commands
        &CAT, &PREFIX, &SUFFIX, &QUOTE, &TRIM, &LOWER, &UPPER, &ASCII, // Map commands
        &FIRST, &SKIP, // Filter commands
        &STREAM, &LOOP, &SEQ, &RAND, &UUID,     // Generator commands
        &HELP_CMD, // Helper commands
    ])
    .done();

fn main() {
    REW.parse_args().run();
}
