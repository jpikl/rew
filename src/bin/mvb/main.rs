use crate::cli::Cli;
use common::input::{Delimiter, PathDiff};
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};
use std::io::Write;

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

    while let Some((in_path, out_path)) = path_diff.read()? {
        writeln!(
            &mut io.stdout(),
            "Moving '{}' to '{}'",
            in_path.to_string_lossy(),
            out_path.to_string_lossy()
        )?;
    }

    Ok(EXIT_CODE_OK)
}
