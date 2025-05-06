use crate::cli::Args;
use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::ErrorKind;
use crate::cli::HELP;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::FILTER_COMMANDS;
use crate::global::NULL;
use crate::run::Context;
use bstr::ByteSlice;
use std::io::copy;

const COUNT: Pos<u128> = PosBuilder::new("count")
    .name("COUNT")
    .description("Number of lines to skip.")
    .required()
    .done();

pub const SKIP: Command = CommandBuilder::new()
    .name("skip")
    .description("Skip first N lines in output.")
    .group(&FILTER_COMMANDS)
    .options(&[HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .positionals(&[COUNT.arg])
    .run(run)
    .done();

fn run(args: Args) -> Result<(), ErrorKind<'static>> {
    let mut count = args.get(&COUNT);
    let context = Context::new(&args);

    if count == 0 {
        copy(&mut context.raw_reader(), &mut context.raw_writer())?;
        return Ok(());
    }

    let separator = context.separator();
    let mut reader = context.byte_chunk_reader();
    let mut writer = context.writer();

    while let Some(chunk) = reader.read_chunk()? {
        let mut start: usize = 0;

        while let Some(end) = chunk[start..].find_byte(separator) {
            start += end + 1;
            count -= 1;

            if count == 0 {
                break;
            }
        }

        if count == 0 {
            writer.write(&chunk[start..])?;
            break;
        }
    }

    writer.write_all_from(reader.get_mut())?;
    Ok(())
}
