use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::common_options;
use crate::global::GENERATOR_COMMANDS;
use crate::run::ContextExt;
use std::io::Read;
use std::io::copy;

const MAX_BUF_SIZE: usize = 128 * 1024 * 1024;

const COUNT: Pos<u64> = PosBuilder::new()
    .name("COUNT")
    .description("Limit for the repetition.")
    .done();

pub const LOOP: Command = CommandBuilder::new()
    .name("loop")
    .description("Output repeatedly all input.")
    .group(&GENERATOR_COMMANDS)
    .options(common_options![])
    .positionals(&[&COUNT.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let count = ctx.args.get_opt(&COUNT);

    if count == Some(0) {
        // No need to read input at all
        return Ok(());
    }

    if count == Some(1) {
        // Avoid buffering the whole input if there is only one output iteration
        copy(&mut ctx.raw_reader(), &mut ctx.raw_writer())?;
        return Ok(());
    }

    let mut reader = ctx.raw_reader();
    let mut writer = ctx.writer();
    let mut completed_bufs: Vec<Vec<u8>> = Vec::new();
    let mut buf = ctx.zeroed_buf(); // Start with small buffer size
    let mut end = 0;

    loop {
        let len = reader.read(&mut buf[end..])?;
        if len == 0 {
            break;
        }

        // Write the first output iteration as we read the input
        writer.write(&buf[end..][..len])?;
        end += len;

        if end < buf.len() {
            continue;
        }

        if buf.len() >= MAX_BUF_SIZE {
            completed_bufs.push(buf);
            // Allocate the new buffer with full capacity to avoid further resizing
            buf = vec![0u8; MAX_BUF_SIZE];
            end = 0;
        } else {
            let new_size = MAX_BUF_SIZE.min(buf.len() << 1);
            buf.resize(new_size, 0);
        }
    }

    if let Some(mut count) = count {
        // We already did first output iteration during reading phase
        while count > 1 {
            for buf in &completed_bufs {
                writer.write(buf)?;
            }
            writer.write(&buf[..end])?;
            count -= 1;
        }
        return Ok(());
    }

    loop {
        for buf in &completed_bufs {
            writer.write(buf)?;
        }
        writer.write(&buf[..end])?;
    }
}
