use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BlockReader;
use crate::io::LineConfig;
use crate::io::Writer;
use anyhow::Result;
use memchr::memchr;

pub const META: Meta = command_meta! {
    name: "first",
    args: Args,
    run: run,
};

/// Output first N input lines.
#[derive(clap::Args)]
struct Args {
    /// Number of lines to print.
    #[arg(default_value_t = 1)]
    count: u128,
}

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let mut count = args.count;

    if count == 0 {
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
                let len = block.len() - remainder.len();
                writer.write_block(&block[..len])?;
                return Ok(());
            }
        }

        writer.write_block(block)?;
    }

    Ok(())
}
