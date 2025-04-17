mod cli;
mod colors;
mod commands;
mod global;
mod io;
mod run;
mod utils;

use anstream::eprintln;
use cli::Command;
use cli::CommandBuilder;
use cli::HELP;
use cli::Parser;
use cli::VERSION;
use commands::cat::CAT;
use commands::prefix::PREFIX;
use commands::quote::QUOTE;
use commands::suffix::SUFFIX;
use global::BUF_MODE;
use global::BUF_SIZE;
use global::NULL;
use std::env::args_os;
use std::process::exit;

const REW: Command = CommandBuilder::new()
    .name(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .version(env!("CARGO_PKG_VERSION"))
    .options(&[HELP.arg, VERSION.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .subcommands(&[CAT, PREFIX, SUFFIX, QUOTE])
    .done();

fn main() {
    let parser = Parser::new(&REW, args_os());

    let runner = match parser.parse() {
        Ok(runner) => runner,
        Err(err) => {
            eprintln!("{err}");
            exit(2);
        }
    };

    match runner.run() {
        Ok(()) => exit(0),
        Err(err) => {
            eprintln!("{err}");
            exit(1)
        }
    }
}
