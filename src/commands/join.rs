use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BlockReader;
use crate::io::LineConfig;
use crate::io::Writer;
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

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let mut reader = BlockReader::from_stdin(global_args);
    let mut writer = Writer::from_stdout(global_args);

    let trim_sparator = global_args.line_separator().trim_fn();
    let input_separator = global_args.line_separator().as_byte();
    let output_separator = args.separator.as_bytes();

    let mut print_separator_before = false;

    while let Some(mut block) = reader.read_block()? {
        while let Some(pos) = memchr(input_separator, block) {
            if print_separator_before {
                writer.write_block(output_separator)?;
            } else {
                print_separator_before = true;
            }
            writer.write_block(trim_sparator(&block[..=pos]))?;
            block = &mut block[(pos + 1)..];
        }

        if !block.is_empty() {
            if print_separator_before {
                writer.write_block(output_separator)?;
            } else {
                print_separator_before = true;
            }
            writer.write_block(block)?;
        }
    }

    if args.trailing {
        writer.write_block(output_separator)?;
    }

    writer.write_block(&[input_separator])
}
