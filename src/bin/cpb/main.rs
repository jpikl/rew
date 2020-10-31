use crate::cli::Cli;
use common::fs::{transfer_path, TransferMode};
use common::input::{Delimiter, PathDiff};
use common::output::{write_error, Log};
use common::run::{exec_run, Io, Result, EXIT_CODE_IO_ERROR, EXIT_CODE_OK};

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
    let mut exit_code = EXIT_CODE_OK;

    while let Some((src_path, dst_path)) = path_diff.read()? {
        if cli.verbose {
            log.begin_copy(&src_path, &dst_path)?;
        }

        match transfer_path(&src_path, &dst_path, TransferMode::Copy) {
            Ok(()) => {
                if cli.verbose {
                    log.end_with_success()?;
                }
            }
            Err(error) => {
                if cli.verbose {
                    log.end_with_failure()?;
                }

                write_error(&mut io.stderr(), &error)?;

                if cli.fail_at_end {
                    exit_code = EXIT_CODE_IO_ERROR;
                } else {
                    return Ok(EXIT_CODE_IO_ERROR);
                }
            }
        }
    }

    Ok(exit_code)
}
