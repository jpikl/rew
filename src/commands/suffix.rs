use crate::cli::Args;
use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::HELP;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::NULL;
use crate::run::Context;
use crate::utils::into_bytes;
use std::ffi::OsString;

const DELETE: Flag = FlagBuilder::new("delete")
    .short('d')
    .long("delete")
    .description("Delete existing suffix intead.")
    .done();

const VALUE: Pos<OsString> = PosBuilder::new("value")
    .name("VALUE")
    .description("Suffix value.")
    .done();

pub const SUFFIX: Command = CommandBuilder::new()
    .name("suffix")
    .description("Add suffix to each line.")
    .options(&[DELETE.arg, HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .positionals(&[VALUE.arg])
    .run(run)
    .done();

fn run(mut args: Args) -> anyhow::Result<()> {
    let delete = args.get(&DELETE);
    let value = into_bytes(args.get_owned(&VALUE));

    let context = Context::new(&args);
    let mut reader = context.line_reader();
    let mut writer = context.writer();

    while let Some(line) = reader.read_line()? {
        if delete {
            writer.write_line(line.strip_suffix(value.as_slice()).unwrap_or(line))?;
        } else {
            writer.write(line)?;
            writer.write(&value)?;
            writer.write_separator()?;
        }
    }

    Ok(())
}
