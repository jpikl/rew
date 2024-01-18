use crate::args::GlobalArgs;
use crate::command::CommandMeta;
use crate::command_meta;
use crate::io::Processing;
use crate::io::Reader;
use crate::io::Writer;
use crate::io::OPTIMAL_IO_BUF_SIZE;
use anyhow::Result;
use bstr::ByteSlice;
use bstr::ByteVec;
use unidecode::unidecode_char;

pub const META: CommandMeta = command_meta! {
    name: "ascii",
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

fn run(global_args: GlobalArgs, args: Args) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);
    let mut buffer = Vec::with_capacity(OPTIMAL_IO_BUF_SIZE);

    reader.for_each_block(|block| {
        if block.is_ascii() {
            // ASCII check is cheap, optimize throughput by reusing input buffer
            writer.write_block(block)?;
            return Ok(Processing::Continue);
        }

        // Copying chars to a side buffer is faster then directly writing them to buffered writer
        if args.delete {
            block
                .chars()
                .filter(|char| char.is_ascii())
                .for_each(|char| buffer.push(char as u8));
        } else {
            block
                .chars()
                .map(unidecode_char)
                .for_each(|str| buffer.push_str(str));
        }

        writer.write_block(&buffer)?;
        buffer.clear();

        Ok(Processing::Continue)
    })
}
