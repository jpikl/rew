use crate::args::GlobalArgs;
use crate::command::CommandMeta;
use crate::command_meta;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Separator;
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
    let separator = Separator::from(&global_args).as_byte();
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);
    let mut count = args.count;

    if count == 0 {
        return Ok(());
    }

    // This is noticably faster than counting lines using `.for_each_line`
    reader.for_each_block(|block| {
        let mut remainder: &[u8] = block;

        while let Some(end) = memchr::memchr(separator, remainder) {
            remainder = &remainder[(end + 1)..];
            count -= 1;

            if count == 0 {
                let len = block.len() - remainder.len();
                writer.write_block(&block[..len])?;
                return Ok(Processing::Abort);
            }
        }

        writer.write_block(block)?;
        Ok(Processing::Continue)
    })
}
