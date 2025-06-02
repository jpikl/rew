use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::HELP;
use crate::global::NULL;
use crate::global::PATH_COMMANDS;
use crate::run::ContextExt;
use crate::utils::path_from_io_bytes;

const WITH_DOT: Flag = FlagBuilder::new("with-dot")
    .short('d')
    .long("with-dot")
    .description("Preserve dot before the extension.")
    .done();

pub const EXT: Command = CommandBuilder::new()
    .name("ext")
    .description("Output extension of paths.")
    .group(&PATH_COMMANDS)
    .options(&[WITH_DOT.arg, HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let with_dot = ctx.args.get(&WITH_DOT);

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        let path = path_from_io_bytes(line)?;

        if let Some(ext) = path.extension() {
            if with_dot {
                writer.write(b".")?;
            }
            writer.write(ext.as_encoded_bytes())?;
        }

        writer.write_separator()?;
    }

    Ok(())
}
