use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::common_options;
use crate::global::MAP_COMMANDS;
use crate::run::ContextExt;
use bstr::ByteSlice;
use bstr::ByteVec;
use deunicode::deunicode_char;

const DELETE: Flag = FlagBuilder::new()
    .short('d')
    .long("delete")
    .description("Delete non-ASCII characters instead of transliteration")
    .done();

pub const ASCII: Command = CommandBuilder::new()
    .name("ascii")
    .description("Transliterate UTF-8 to ASCII.")
    .group(&MAP_COMMANDS)
    .options(common_options![&DELETE.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let delete = ctx.args.get(&DELETE);

    let mut reader = ctx.char_chunk_reader();
    let mut writer = ctx.writer();
    let mut buffer = ctx.uninit_buf();

    while let Some(chunk) = reader.read_chunk()? {
        if chunk.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            writer.write(chunk)?;
            continue;
        }

        // Copying chars to a side buffer is faster than directly writing them to buffered writer
        for char in chunk.chars() {
            if char.is_ascii() {
                buffer.push(char as u8);
            } else if !delete {
                buffer.push_str(deunicode_char(char).unwrap_or("?").as_bytes());
            }
        }

        writer.write(&buffer)?;
        buffer.clear();
    }

    Ok(())
}
