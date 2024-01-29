use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BlockReader;
use crate::io::BufSizeConfig;
use crate::io::Writer;
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

fn run(global_args: &GlobalArgs, _args: &Args) -> Result<()> {
    let mut reader = BlockReader::from_stdin(global_args);
    let mut writer = Writer::from_stdout(global_args);
    let mut buffer = Vec::with_capacity(global_args.buf_size());

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
