use cli::Cli;
use common::run::{exec_run, Io, Result};
use common::transfer::{run_transfer, TransferMode};

mod cli;

fn main() {
    exec_run(run);
}

fn run(cli: &Cli, io: &Io) -> Result {
    run_transfer(cli, io, TransferMode::Copy)
}
