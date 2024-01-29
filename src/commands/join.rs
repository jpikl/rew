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

    let trim_sparator = context.separator().trim_fn();
    let input_separator = context.separator().as_byte();
    let output_separator = args.separator.as_bytes();

    let mut separate_next_block = false;

    while let Some(mut block) = reader.read_block()? {
        while let Some(pos) = memchr(input_separator, block) {
            if separate_next_block {
                writer.write_block(output_separator)?;
            } else {
                separate_next_block = true;
            }
            writer.write_block(trim_sparator(&block[..=pos]))?;
            block = &mut block[(pos + 1)..];
        }

        if !block.is_empty() {
            if separate_next_block {
                writer.write_block(output_separator)?;
                separate_next_block = false;
            }
            writer.write_block(block)?;
        }
    }

    if args.trailing {
        writer.write_block(output_separator)?;
    }

    writer.write_block(&[input_separator])
}
