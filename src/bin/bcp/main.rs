use crate::cli::Cli;
use common::io::Io;
use common::run::{Result, Runner};

mod cli;

fn main() {
    Runner::new().exec(run)
}

fn run<'a, IO: Io<'a>>(_cli: &'a Cli, _io: &'a IO) -> Result {
    unimplemented!()
}
