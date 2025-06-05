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
    .options(common_options![])
    .positionals(&[&COUNT.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut count = ctx.args.get(&COUNT);

    if count == 0 {
        copy(&mut ctx.raw_reader(), &mut ctx.raw_writer())?;
        return Ok(());
    }

    let separator = ctx.separator();
    let mut reader = ctx.byte_chunk_reader();
    let mut writer = ctx.writer();

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
