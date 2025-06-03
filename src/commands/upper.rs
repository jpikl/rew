use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::common_options;
use crate::global::MAP_COMMANDS;
use crate::run::ContextExt;
use bstr::ByteSlice;

pub const UPPER: Command = CommandBuilder::new()
    .name("upper")
    .description("Convert characters to uppercase.")
    .group(&MAP_COMMANDS)
    .options(common_options![])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut reader = ctx.char_chunk_reader();
    let mut writer = ctx.writer();
    let mut buffer = ctx.uninit_buf();

    while let Some(chunk) = reader.read_chunk()? {
        if chunk.is_ascii() {
            chunk.make_ascii_uppercase();
            writer.write(chunk)?;
        } else {
            buffer.clear();
            chunk.to_uppercase_into(&mut buffer);
            writer.write(&buffer)?;
        }
    }

    Ok(())
}
