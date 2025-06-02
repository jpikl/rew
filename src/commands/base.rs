use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::HELP;
use crate::global::NULL;
use crate::global::PATH_COMMANDS;
use crate::run::ContextExt;
use crate::utils::path_from_io_bytes;

pub const BASE: Command = CommandBuilder::new()
    .name("base")
    .description("Output base name of paths.")
    .group(&PATH_COMMANDS)
    .options(&[HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        let path = path_from_io_bytes(line)?;
        let stem = path.file_stem().unwrap_or_default();
        writer.write_line(stem.as_encoded_bytes())?;
    }

    Ok(())
}
