use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::common_options;
use crate::global::PATH_COMMANDS;
use crate::path::get_prefix_len;
use crate::run::ContextExt;

pub const DRIVE: Command = CommandBuilder::new()
    .name("drive")
    .description("Output drive of each path.")
    .description_ex(&[
        "For Windows-style paths, the output are  prefixes like `C:` or `\\\\?\\server\\share`.",
        "For Unix-style paths, the output is always empty.",
    ])
    .group(&PATH_COMMANDS)
    .options(common_options![])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();
    let path_style = ctx.path_style();

    while let Some(line) = reader.read_line()? {
        let len = get_prefix_len(line, path_style);
        writer.write_line(&line[..len])?;
    }

    Ok(())
}
