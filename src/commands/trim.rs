use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Writer;
use anyhow::Result;
use bstr::ByteSlice;

pub const META: Meta = command_meta! {
    name: "trim",
    args: Args,
    run: run,
};

/// Trim whitespaces from each line.
///
/// By default, both the beginning and the end are trimmed.
#[derive(clap::Args)]
struct Args {
    /// Trim the beginning.
    #[arg(short, long)]
    start: bool,

    /// Trim the end.
    #[arg(short, long)]
    end: bool,
}

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let mut reader = Reader::from(global_args);
    let mut writer = Writer::from(global_args);

    reader.for_each_line(|line| {
        let result = match (args.start, args.end) {
            (true, true) | (false, false) => line.trim(),
            (true, false) => line.trim_start(),
            (false, true) => line.trim_end(),
        };
        writer.write_line(result)?;
        Ok(Processing::Continue)
    })
}
