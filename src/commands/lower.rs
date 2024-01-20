use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Writer;
use crate::io::OPTIMAL_IO_BUF_SIZE;
use anyhow::Result;
use bstr::ByteSlice;

pub const META: Meta = command_meta! {
    name: "lower",
    args: Args,
    run: run,
};

/// Convert characters to lowercase.
#[derive(clap::Args)]
struct Args;

fn run(global_args: &GlobalArgs, _args: &Args) -> Result<()> {
    let mut reader = Reader::from(global_args);
    let mut writer = Writer::from(global_args);
    let mut buffer = Vec::with_capacity(OPTIMAL_IO_BUF_SIZE);

    reader.for_each_block(|block| {
        if block.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            block.make_ascii_lowercase();
            writer.write_block(block)?;
        } else {
            buffer.clear();
            block.to_lowercase_into(&mut buffer);
            writer.write_block(&buffer)?;
        }
        Ok(Processing::Continue)
    })
}
