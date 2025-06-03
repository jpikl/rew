use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::common_options;
use crate::global::PATH_COMMANDS;
use crate::run::ContextExt;
use crate::utils::path_from_io_bytes;

pub const WITH_DIR: Flag = FlagBuilder::new("full")
    .short('d')
    .long("with-dir")
    .description("Keep directory path before the base name.")
    .description_ex(&["This has basically the same effect as just removing dot and extension from the path."])
    .done();

pub const BASE: Command = CommandBuilder::new()
    .name("base")
    .description("Output base name of paths.")
    .group(&PATH_COMMANDS)
    .options(common_options![WITH_DIR.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let with_dir = ctx.args.get(&WITH_DIR);

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        let path = path_from_io_bytes(line)?;

        if with_dir {
            if let Some(ext) = path.extension() {
                let end = line.len() - ext.len() - 1;
                writer.write_line(&line[..end])?;
            } else {
                writer.write_line(line)?;
            }
        } else {
            let stem = path.file_stem().unwrap_or_default();
            writer.write_line(stem.as_encoded_bytes())?;
        }
    }

    Ok(())
}
