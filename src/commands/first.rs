use crate::args::GlobalArgs;
use crate::command::CommandMeta;
use crate::command_meta;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Writer;
use anyhow::Result;

pub const META: CommandMeta = command_meta! {
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

fn run(global_args: GlobalArgs, args: Args) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);
    let mut count = args.count;

    if count == 0 {
        return Ok(());
    }

    reader.for_each_line(|line| {
        writer.write_line(line)?;
        count -= 1;

        if count > 0 {
            Ok(Processing::Continue)
        } else {
            Ok(Processing::Abort)
        }
    })
}
