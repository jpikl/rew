use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use bstr::ByteSlice;
use bstr::ByteVec;
use unidecode::unidecode_char;

pub const META: Meta = command_meta! {
    name: "ascii",
    group: Group::Mappers,
    args: Args,
    run: run,
};

/// Convert characters to ASCII.
#[derive(clap::Args)]
struct Args {
    /// Delete non-ASCII characters instead of converting them.
    #[arg(short, long)]
    delete: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut reader = context.block_reader();
    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    while let Some(block) = reader.read_block()? {
        if block.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            writer.write_block(block)?;
            continue;
        }

        // Copying chars to a side buffer is faster then directly writing them to buffered writer
        if args.delete {
            block
                .chars()
                .filter(char::is_ascii)
                .for_each(|char| buffer.push(char as u8));
        } else {
            block
                .chars()
                .map(unidecode_char)
                .for_each(|str| buffer.push_str(str));
        }

        writer.write_block(&buffer)?;
        buffer.clear();
    }

    Ok(())
}
