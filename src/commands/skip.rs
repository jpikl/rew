use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use memchr::memchr;
use std::io::copy;

pub const META: Meta = command_meta! {
    name: "skip",
    group: Group::Filters,
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

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut count = args.count;

    if count == 0 {
        copy(&mut context.raw_reader(), &mut context.raw_writer())?;
        return Ok(());
    }

    let mut reader = context.block_reader();
    let mut writer = context.writer();
    let separator = context.separator().as_byte();

    while let Some(block) = reader.read_block()? {
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
            break;
        }
    }

    writer.write_all_from(reader.get_mut())
}
