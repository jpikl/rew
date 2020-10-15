use crate::cli::Cli;
use common::io::Io;
use common::run::{Result, Runner, EXIT_CODE_OK};

mod cli;

fn main() {
    Runner::new().exec(run);
}

fn run<'a, IO: Io<'a>>(_cli: &'a Cli, _io: &'a IO) -> Result {
    Ok(EXIT_CODE_OK)
}
