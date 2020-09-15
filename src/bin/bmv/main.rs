use crate::cli::Cli;
use common::run::{exec_run, Result};
use std::io::Stdin;
use structopt::StructOpt;
use termcolor::StandardStream;

mod cli;

fn main() {
    exec_run(run, Cli::from_args());
}

fn run(
    cli: Cli,
    stdin: &mut Stdin,
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
) -> Result {
    unimplemented!()
}
