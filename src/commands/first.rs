use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use memchr::memchr;

pub const META: Meta = command_meta! {
    name: "first",
    group: Group::Filters,
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

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut count = args.count;

    if count == 0 {
        return Ok(());
    }

    let mut reader = context.chunk_reader();
    let mut writer = context.writer();
    let separator = context.separator().as_byte();

    while let Some(chunk) = reader.read_chunk()? {
        let mut remainder: &[u8] = chunk;

        while let Some(end) = memchr(separator, remainder) {
            remainder = &remainder[(end + 1)..];
            count -= 1;

            if count == 0 {
                let len = chunk.len() - remainder.len();
                writer.write(&chunk[..len])?;
                return Ok(());
            }
        }

        writer.write(chunk)?;
    }

    Ok(())
}
