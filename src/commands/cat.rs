use crate::cli::Args;
use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::HELP;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::NULL;
use crate::run::Context;
use std::io::copy;

const LINES: Flag = FlagBuilder::new("lines")
    .short('l')
    .long("lines")
    .description("Process data as lines.")
    .description_ex(&["Will normalize newlines to LF as a side effect."])
    .done();

const CHARS: Flag = FlagBuilder::new("chars")
    .short('c')
    .long("chars")
    .description("Process data as character chunks.")
    .done();

const BYTES: Flag = FlagBuilder::new("bytes")
    .short('b')
    .long("bytes")
    .description("Process data as byte chunks.")
    .done();

pub const CAT: Command = CommandBuilder::new()
    .name("cat")
    .description("Copy all input to output.")
    .description_ex(&["Mostly useful for benchmarking raw IO throughput."])
    .options(&[
        LINES.arg,
        CHARS.arg,
        BYTES.arg,
        HELP.arg,
        NULL.arg,
        BUF_SIZE.arg,
        BUF_MODE.arg,
    ])
    .run(run)
    .done();

fn run(args: Args) -> Result<(), ErrorKind<'static>> {
    let lines = args.get(&LINES);
    let chars = args.get(&CHARS);
    let bytes = args.get(&BYTES);

    if (lines as u8 + chars as u8 + bytes as u8) > 1 {
        return Err(ErrorKind::MutuallyExclusiveOptions(&[
            &LINES.arg, &CHARS.arg, &BYTES.arg,
        ]));
    }

    let context = Context::new(&args);

    if lines {
        let mut reader = context.line_reader();
        let mut writer = context.writer();

        while let Some(line) = reader.read_line()? {
            writer.write_line(line)?;
        }
    } else if chars {
        let mut reader = context.char_chunk_reader();
        let mut writer = context.writer();

        while let Some(chunk) = reader.read_chunk()? {
            writer.write(chunk)?;
        }
    } else if bytes {
        let mut reader = context.byte_chunk_reader();
        let mut writer = context.writer();

        while let Some(chunk) = reader.read_chunk()? {
            writer.write(chunk)?;
        }
    } else {
        let mut reader = context.raw_reader();
        let mut writer = context.raw_writer();

        copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
