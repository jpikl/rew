use crate::cli::Cli;
use common::input::{Delimiter, PathDiff};
use common::output::Log;
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};

mod cli;

fn main() {
    exec_run(run);
}

fn run(cli: &Cli, io: &Io) -> Result {
    let delimiter = if cli.read_nul {
        Delimiter::Nul
    } else {
        Delimiter::Newline
    };

    let mut path_diff = PathDiff::new(io.stdin(), delimiter);
    let mut log = Log::new(io.stdout());

    while let Some((src_path, dst_path)) = path_diff.read()? {
        if cli.verbose {
            log.begin_move(&src_path, &dst_path)?;
            log.end_with_success()?;
        }
    }

    Ok(EXIT_CODE_OK)
}
