use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use memchr::memchr;

pub const META: Meta = command_meta! {
    name: "join",
    group: Group::Transformers,
    args: Args,
    run: run,
};

/// Join input lines using a separator.
#[derive(clap::Args)]
struct Args {
    //// Separator.
    separator: String,

    /// Print trailing separator at the end.
    #[arg(short = 't', long)]
    trailing: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut reader = context.block_reader();
    let mut writer = context.writer();

    let output_separator = args.separator.as_bytes();
    let input_separator = context.separator().as_byte();
    let trim_input_sparator = context.separator().trim_fn();

    let mut start_next_item_separated = false;

    while let Some(block) = reader.read_block()? {
        let mut remainder: &[u8] = block;

        while let Some(pos) = memchr(input_separator, remainder) {
            if start_next_item_separated {
                writer.write_block(output_separator)?;
            }

            writer.write_block(trim_input_sparator(&remainder[..=pos]))?;
            remainder = &remainder[(pos + 1)..];

            // Write the trimmed separator once we know there is more data
            start_next_item_separated = true;
        }

        if !remainder.is_empty() {
            if start_next_item_separated {
                writer.write_block(output_separator)?;
            }

            writer.write_block(remainder)?;
            start_next_item_separated = false;
        }
    }

    if args.trailing {
        writer.write_block(output_separator)?;
    }

    writer.write_block(&[input_separator])
}
