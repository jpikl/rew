use crate::cli::Cli;
use common::run::{exec_run, Result};
use std::io::Stdin;
use termcolor::StandardStream;

mod cli;

fn main() {
    exec_run(run);
}

fn run(
    _cli: Cli,
    _stdin: &mut Stdin,
    _stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
) -> Result {
    unimplemented!()
}
