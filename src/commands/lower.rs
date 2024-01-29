use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use bstr::ByteSlice;

pub const META: Meta = command_meta! {
    name: "lower",
    group: Group::Mappers,
    args: Args,
    run: run,
};

/// Convert characters to lowercase.
#[derive(clap::Args)]
struct Args;

fn run(context: &Context, _args: &Args) -> Result<()> {
    let mut reader = context.block_reader();
    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    while let Some(block) = reader.read_block()? {
        if block.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            block.make_ascii_lowercase();
            writer.write_block(block)?;
        } else {
            buffer.clear();
            block.to_lowercase_into(&mut buffer);
            writer.write_block(&buffer)?;
        }
    }

    Ok(())
}
