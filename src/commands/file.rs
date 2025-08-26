use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::common_options;
use crate::global::PATH_COMMANDS;
use crate::path::get_file;
use crate::run::ContextExt;
use crate::utils::path_from_io_bytes;
use std::ffi::OsStr;
use std::path::Component;

pub const FILE: Command = CommandBuilder::new()
    .name("file")
    .description("Output the last component of each path.")
    .description_ex(&["For path like `/root/dir/name.ext`, output `name.ext`."])
    .group(&PATH_COMMANDS)
    .options(common_options![])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();
    let path_style = ctx.path_style();

    while let Some(path) = reader.read_line()? {
        writer.write_line(get_file(path, path_style))?;
    }

    Ok(())
}
