use crate::args::GlobalArgs;
use crate::command::CommandMeta;
use crate::command_meta;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Writer;
use anyhow::Result;

pub const META: CommandMeta = command_meta! {
    name: "cat",
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
    #[arg(short, long)]
    lines: bool,
}

fn run(global_args: GlobalArgs, args: Args) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);

    if args.lines {
        reader.for_each_line(|line| {
            writer.write_line(line)?;
            Ok(Processing::Continue)
        })
    } else {
        reader.for_each_block(|block| {
            writer.write_block(block)?;
            Ok(Processing::Continue)
        })
    }
}
