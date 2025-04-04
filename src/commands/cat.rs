use crate::cli::Args;
use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::HELP;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::NULL;
use crate::run::Context;
use anyhow::bail;
use bstr::ByteSlice;

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

fn run(args: Args) -> anyhow::Result<()> {
    let lines = args.get(&LINES);
    let chars = args.get(&CHARS);
    let bytes = args.get(&BYTES);

    if (lines as u8 + chars as u8 + bytes as u8) > 1 {
        bail!("Options {LINES} / {CHARS} / {BYTES} are mutually exclusive.");
    }

    let context = Context::new(&args);

    if lines {
        let mut reader = context.line_reader();

        while let Some(line) = reader.read_line()? {
            // TODO optimize write
            println!("{}", line.to_str_lossy());
        }
    } else if chars {
        unimplemented!("chars not implemented yet");
    } else if bytes {
        unimplemented!("bytes not implemented yet");
    }  else {
        unimplemented!("default mode implemented yet");
    }

    Ok(())
}
