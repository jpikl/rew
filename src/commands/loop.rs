use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::Writer;
use crate::io::OPTIMAL_IO_BUF_SIZE;
use anyhow::Result;
use std::io::copy;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;

pub const META: Meta = command_meta! {
    name: "loop",
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

    let mut reader = stdin().lock();
    let mut writer = Writer::from_stdout(global_args);
    let mut buffer = vec![0; OPTIMAL_IO_BUF_SIZE];
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

        if buffer.len() - end < OPTIMAL_IO_BUF_SIZE / 2 {
            buffer.resize(buffer.len() + OPTIMAL_IO_BUF_SIZE, 0);
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
