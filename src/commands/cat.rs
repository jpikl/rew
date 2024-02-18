use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use anyhow::Result;
use std::io::copy;

pub const META: Meta = command_meta! {
    name: "cat",
    group: Group::Mappers,
    args: Args,
    run: run,
    examples: command_examples! [],
};

/// Copy all input to output
///
/// Mostly useful for benchmarking raw IO throughput of rew.
#[derive(clap::Args)]
struct Args {
    /// Process data as lines.
    ///
    /// Will normalize newlines to LF as a side-effect.
    #[arg(short, long, conflicts_with = "chunks")]
    lines: bool,

    /// Process data as chunks.
    #[arg(short, long, conflicts_with = "lines")]
    chunks: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    if args.lines {
        let mut reader = context.line_reader();
        let mut writer = context.writer();

        while let Some(line) = reader.read_line()? {
            writer.write_line(line)?;
        }
    } else if args.chunks {
        let mut reader = context.chunk_reader();
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
