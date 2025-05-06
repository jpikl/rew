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

const COUNT: Pos<u128> = PosBuilder::new("count")
    .name("COUNT")
    .description("Number of lines to print.")
    .default("1")
    .done();

pub const FIRST: Command = CommandBuilder::new()
    .name("first")
    .description("Output first N lines.")
    .group(&FILTER_COMMANDS)
    .options(&[HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .positionals(&[COUNT.arg])
    .run(run)
    .done();

fn run(args: Args) -> Result<(), ErrorKind<'static>> {
    let mut count = args.get(&COUNT);

    if count == 0 {
        return Ok(());
    }

    let context = Context::new(&args);
    let separator = context.separator();

    let mut reader = context.byte_chunk_reader();
    let mut writer = context.writer();

    while let Some(chunk) = reader.read_chunk()? {
        let mut start: usize = 0;

        while let Some(pos) = chunk[start..].find_byte(separator) {
            start += pos + 1;
            count -= 1;

            if count == 0 {
                writer.write(&chunk[..start])?;
                return Ok(());
            }
        }

        writer.write(chunk)?;
    }

    Ok(())
}
