use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::common_options;
use crate::global::MAP_COMMANDS;
use crate::run::ContextExt;
use std::io::copy;

const LINES: Flag = FlagBuilder::new()
    .short('l')
    .long("lines")
    .description("Process data as lines.")
    .description_ex(&["Will normalize newlines to LF as a side effect."])
    .done();

const CHARS: Flag = FlagBuilder::new()
    .short('c')
    .long("chars")
    .description("Process data as character chunks.")
    .done();

const BYTES: Flag = FlagBuilder::new()
    .short('b')
    .long("bytes")
    .description("Process data as byte chunks.")
    .done();

pub const CAT: Command = CommandBuilder::new()
    .name("cat")
    .description("Copy input data to output.")
    .description_ex(&["Mostly useful for benchmarking raw IO throughput."])
    .group(&MAP_COMMANDS)
    .options(common_options![&LINES.arg, &CHARS.arg, &BYTES.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let lines = ctx.args.get(&LINES);
    let chars = ctx.args.get(&CHARS);
    let bytes = ctx.args.get(&BYTES);

    if (lines as u8 + chars as u8 + bytes as u8) > 1 {
        return Err(ErrorKind::MutuallyExclusiveOptions(vec![
            &LINES.arg, &CHARS.arg, &BYTES.arg,
        ]));
    }

    if lines {
        let mut reader = ctx.line_reader();
        let mut writer = ctx.writer();

        while let Some(line) = reader.read_line()? {
            writer.write_line(line)?;
        }
    } else if chars {
        let mut reader = ctx.char_chunk_reader();
        let mut writer = ctx.writer();

        while let Some(chunk) = reader.read_chunk()? {
            writer.write(chunk)?;
        }
    } else if bytes {
        let mut reader = ctx.byte_chunk_reader();
        let mut writer = ctx.writer();

        while let Some(chunk) = reader.read_chunk()? {
            writer.write(chunk)?;
        }
    } else {
        let mut reader = ctx.raw_reader();
        let mut writer = ctx.raw_writer();

        copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
