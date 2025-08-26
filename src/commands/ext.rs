use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::common_options;
use crate::global::PATH_COMMANDS;
use crate::path::get_extension;
use crate::run::ContextExt;
use crate::utils::path_from_io_bytes;

const WITH_DOT: Flag = FlagBuilder::new()
    .short('d')
    .long("with-dot")
    .description("Keep `.` before the extension.")
    .description_ex(&["For path like `name.ext`, output `.ext` instead of `ext`."])
    .done();

const LONGEST: Flag = FlagBuilder::new()
    .short('l')
    .long("longest")
    .description("Try to match the longest possible extension (after the first `.`).")
    .description_ex(&["For path like `archive.tar.gz`, output `tar.gz` instead of `gz`."])
    .done();

pub const EXT: Command = CommandBuilder::new()
    .name("ext")
    .description("Output extension of each path.")
    .description_ex(&["For path like `/root/dir/name.ext`, output `ext`."])
    .group(&PATH_COMMANDS)
    .options(common_options![&WITH_DOT.arg, &LONGEST.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let with_dot = ctx.args.get(&WITH_DOT);
    let longest = ctx.args.get(&LONGEST);
    let path_style = ctx.path_style();

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(path) = reader.read_line()? {
        writer.write_line(get_extension(path, path_style, with_dot, longest))?;
    }

    Ok(())
}
