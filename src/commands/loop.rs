use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::BufSizeConfig;
use crate::io::Writer;
use anyhow::Result;
use std::io::copy;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;

pub const META: Meta = command_meta! {
    name: "loop",
    group: Group::Transformers,
    args: Args,
    run: run,
};

/// Repeatedly output all captured input.
#[derive(clap::Args)]
struct Args {
    /// How many times do the repetition (default: forever).
    #[arg()]
    count: Option<u128>,
}

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let count = args.count;

    if count == Some(0) {
        return Ok(());
    }

    if count == Some(1) {
        // Avoid buffering the whole input if there is only one output iteration
        copy(&mut stdin().lock(), &mut stdout().lock())?;
        return Ok(());
    }

    let buf_size = global_args.buf_size();
    let mut reader = stdin().lock();
    let mut writer = Writer::from_stdout(global_args);
    let mut buffer = vec![0; buf_size];
    let mut end = 0;

    loop {
        let len = reader.read(&mut buffer[end..])?;
        if len == 0 {
            break;
        }

        // Write the first output iteration as we read the input
        let next_end = end + len;
        writer.write_block(&buffer[end..next_end])?;
        end = next_end;

        if buffer.len() - end < buf_size / 2 {
            buffer.resize(buffer.len() + buf_size, 0);
        }
    }

    if let Some(mut count) = count {
        // We already did first output iteration during reading phase
        while count > 1 {
            writer.write_block(&buffer[..end])?;
            count -= 1;
        }
        return Ok(());
    }

    loop {
        writer.write_block(&buffer[..end])?;
    }
}
