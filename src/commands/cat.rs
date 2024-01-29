use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BlockReader;
use crate::io::LineReader;
use crate::io::Writer;
use anyhow::Result;
use std::io::copy;
use std::io::stdin;
use std::io::stdout;

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

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    if args.lines {
        let mut reader = LineReader::from_stdin(global_args);
        let mut writer = Writer::from_stdout(global_args);

        while let Some(line) = reader.read_line()? {
            writer.write_line(line)?;
        }
    } else if args.blocks {
        let mut reader = BlockReader::from_stdin(global_args);
        let mut writer = Writer::from_stdout(global_args);

        while let Some(block) = reader.read_block()? {
            writer.write_block(block)?;
        }
    } else {
        let mut reader = stdin().lock();
        let mut writer = stdout().lock();

        copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
