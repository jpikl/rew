use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::common_options;
use crate::global::FILTER_COMMANDS;
use crate::run::ContextExt;
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
    .options(common_options![])
    .positionals(&[COUNT.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut count = ctx.args.get(&COUNT);

    if count == 0 {
        return Ok(());
    }

    let separator = ctx.separator();
    let mut reader = ctx.byte_chunk_reader();
    let mut writer = ctx.writer();

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
