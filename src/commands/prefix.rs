use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::HELP;
use crate::global::MAP_COMMANDS;
use crate::global::NULL;
use crate::run::ContextExt;
use bstr::BString;

const DELETE: Flag = FlagBuilder::new("delete")
    .short('d')
    .long("delete")
    .description("Delete existing prefix intead.")
    .done();

const VALUE: Pos<BString> = PosBuilder::new("values")
    .name("VALUE")
    .description("Prefix value.")
    .required()
    .done();

pub const PREFIX: Command = CommandBuilder::new()
    .name("prefix")
    .description("Add prefix to each line.")
    .group(&MAP_COMMANDS)
    .options(&[DELETE.arg, HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .positionals(&[VALUE.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let delete = ctx.args.get(&DELETE);
    let prefix = ctx.args.get_ref(&VALUE);
    let prefix = prefix.as_slice();

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        if delete {
            writer.write_line(line.strip_prefix(prefix).unwrap_or(line))?;
        } else {
            writer.write(prefix)?;
            writer.write(line)?;
            writer.write_separator()?;
        }
    }

    Ok(())
}
