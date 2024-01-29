use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use std::io::copy;

pub const META: Meta = command_meta! {
    name: "cat",
    group: Group::Mappers,
    args: Args,
    run: run,
};

/// Copy all input to output
///
/// Mostly useful for benchmarking raw IO throughput of rew.
#[derive(clap::Args)]
struct Args {
    /// Process data as lines.
    ///
    /// Will normalize newlines to LF as a side-effect.
    #[arg(short, long, conflicts_with = "blocks")]
    lines: bool,

    /// Process data as blocks.
    #[arg(short, long, conflicts_with = "lines")]
    blocks: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    if args.lines {
        let mut reader = context.line_reader();
        let mut writer = context.writer();

        while let Some(line) = reader.read_line()? {
            writer.write_line(line)?;
        }
    } else if args.blocks {
        let mut reader = context.block_reader();
        let mut writer = context.writer();

        while let Some(block) = reader.read_block()? {
            writer.write_block(block)?;
        }
    } else {
        let mut reader = context.raw_reader();
        let mut writer = context.raw_writer();

        copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
