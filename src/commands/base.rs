use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::common_options;
use crate::global::PATH_COMMANDS;
use crate::path::get_base;
use crate::run::ContextExt;

pub const WITH_DIR: Flag = FlagBuilder::new()
    .short('d')
    .long("with-dir")
    .description("Keep full path before the base name.")
    .description_ex(&["This has basically the same effect as just removing dot and extension from the path."])
    .done();

const SHORTEST: Flag = FlagBuilder::new()
    .short('s')
    .long("shortest")
    .description("Try to match the shortest possible base name (before the first `.`).")
    .description_ex(&["For path like `archive.tar.gz`, output `archive` instead of `archive.tar`."])
    .done();

pub const BASE: Command = CommandBuilder::new()
    .name("base")
    .description("Output the last component of each path (without extension).")
    .description_ex(&["For path like `/root/dir/name.ext`, output `name`."])
    .group(&PATH_COMMANDS)
    .options(common_options![&WITH_DIR.arg, &SHORTEST.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let with_dir = ctx.args.get(&WITH_DIR);
    let shortest = ctx.args.get(&SHORTEST);
    let path_style = ctx.path_style();

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(path) = reader.read_line()? {
        writer.write_line(get_base(path, path_style, with_dir, shortest))?;
    }

    Ok(())
}
