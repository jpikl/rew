use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BlockReader;
use crate::io::Writer;
use crate::io::OPTIMAL_IO_BUF_SIZE;
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

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let mut reader = BlockReader::from_stdin();
    let mut writer = Writer::from_stdout(global_args);
    let mut buffer = Vec::with_capacity(OPTIMAL_IO_BUF_SIZE);

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
