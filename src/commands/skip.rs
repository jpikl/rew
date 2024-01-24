use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::copy_blocks;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Separator;
use crate::io::Writer;
use anyhow::Result;
use memchr::memchr;

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
    let mut reader = Reader::from(global_args);
    let mut writer = Writer::from(global_args);
    let separator = Separator::from(global_args).as_byte();

    if count == 0 {
        return copy_blocks(&mut reader, &mut writer);
    }

    // This is noticably faster than counting lines using `.for_each_line`
    reader.for_each_block(|block| {
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
            Ok(Processing::Abort)
        } else {
            Ok(Processing::Continue)
        }
    })?;

    copy_blocks(&mut reader, &mut writer)
}
