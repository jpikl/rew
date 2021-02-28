use crate::input::Terminator;
use crate::output::write_error;
use crate::run::{Io, Options, Result, EXIT_CODE_IO_ERROR, EXIT_CODE_OK};
use crate::transfer::fs::{transfer_path, TransferMode};
use crate::transfer::input::PathDiff;
use crate::transfer::output::TransferLog;

pub trait TransferOptions {
    fn read_nul(&self) -> bool;
    fn verbose(&self) -> bool;
    fn fail_at_end(&self) -> bool;
}

pub fn run_transfer<O>(options: &O, io: &Io, mode: TransferMode) -> Result
where
    O: Options + TransferOptions,
{
    let terminator = if options.read_nul() {
        Terminator::Byte(0)
    } else {
        Terminator::Newline
    };

    let mut path_diff = PathDiff::new(io.stdin(), terminator);
    let mut log = TransferLog::new(io.stdout());
    let mut exit_code = EXIT_CODE_OK;

    while let Some((src_path, dst_path)) = path_diff.read()? {
        if options.verbose() {
            log.begin_transfer(mode, &src_path, &dst_path)?;
        }

        match transfer_path(&src_path, &dst_path, mode) {
            Ok(()) => {
                if options.verbose() {
                    log.end_with_success()?;
                }
            }
            Err(error) => {
                if options.verbose() {
                    log.end_with_failure()?;
                }

                write_error(&mut io.stderr(), &error)?;

                if options.fail_at_end() {
                    exit_code = EXIT_CODE_IO_ERROR;
                } else {
                    return Ok(EXIT_CODE_IO_ERROR);
                }
            }
        }
    }

    Ok(exit_code)
}
