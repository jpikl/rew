use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::common_options;
use crate::global::PATH_COMMANDS;
use crate::run::ContextExt;
use crate::utils::path_from_io_bytes;
use std::ffi::OsStr;

pub const DIR: Command = CommandBuilder::new()
    .name("dir")
    .description("Strip the last component from each path.")
    .description_ex(&[
        "For path like `/root/dir/name.ext`, output `/root/dir`.",
        "If there is no remaining component, output `.`.",
    ])
    .group(&PATH_COMMANDS)
    .options(common_options![])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        let path = path_from_io_bytes(line)?;

        let parent = match path.parent() {
            Some(parent) => parent.as_os_str(),
            None => OsStr::new(""),
        };

        if parent.is_empty() {
            if path.is_absolute() {
                writer.write_line(path.as_os_str().as_encoded_bytes())?;
            } else {
                writer.write_line(b".")?;
            }
        } else {
            writer.write_line(parent.as_encoded_bytes())?;
        }
    }

    Ok(())
}
