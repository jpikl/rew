use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::common_options;
use crate::global::MAP_COMMANDS;
use crate::run::ContextExt;
use bstr::BString;

const DELETE: Flag = FlagBuilder::new("delete")
    .short('d')
    .long("delete")
    .description("Delete existing suffix intead.")
    .done();

const VALUE: Pos<BString> = PosBuilder::new("values")
    .name("VALUE")
    .description("Suffix value.")
    .required()
    .done();

pub const SUFFIX: Command = CommandBuilder::new()
    .name("suffix")
    .description("Add suffix to each line.")
    .group(&MAP_COMMANDS)
    .options(common_options![DELETE.arg])
    .positionals(&[VALUE.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let delete = ctx.args.get(&DELETE);
    let suffix = ctx.args.get_ref(&VALUE);
    let suffix = suffix.as_slice();

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        if delete {
            writer.write_line(line.strip_suffix(suffix).unwrap_or(line))?;
        } else {
            writer.write(line)?;
            writer.write(suffix)?;
            writer.write_separator()?;
        }
    }

    Ok(())
}
