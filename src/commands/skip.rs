use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BlockReader;
use crate::io::LineConfig;
use crate::io::Writer;
use anyhow::Result;
use memchr::memchr;
use std::io::copy;
use std::io::stdin;
use std::io::stdout;

pub const META: Meta = command_meta! {
    name: "skip",
    args: Args,
    run: run,
};

/// Skip first N input lines, output the rest.
#[derive(clap::Args)]
struct Args {
    /// Number of lines to skip.
    #[arg()]
    count: u128,
}

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let mut count = args.count;

    if count == 0 {
        copy(&mut stdin().lock(), &mut stdout().lock())?;
        return Ok(());
    }

    let mut reader = BlockReader::from_stdin();
    let mut writer = Writer::from_stdout(global_args);
    let separator = global_args.line_separator().as_byte();

    while let Some(block) = reader.read_block()? {
        let mut remainder: &[u8] = block;

        while let Some(end) = memchr(separator, remainder) {
            remainder = &remainder[(end + 1)..];
            count -= 1;

            if count == 0 {
                break;
            }
        }

        if count == 0 {
            writer.write_block(remainder)?;
            break;
        }
    }

    writer.write_all_from(&mut reader)
}
