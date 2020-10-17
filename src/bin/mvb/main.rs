use crate::cli::Cli;
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};

mod cli;

fn main() {
    exec_run(run);
}

fn run(_cli: &Cli, _io: &Io) -> Result {
    Ok(EXIT_CODE_OK)
}
