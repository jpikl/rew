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
use bstr::BString;

const DELETE: Flag = FlagBuilder::new("delete")
    .short('d')
    .long("delete")
    .description("Delete existing prefix intead.")
    .done();

const VALUE: Pos<BString> = PosBuilder::new("value")
    .name("VALUE")
    .description("Prefix value.")
    .done();

pub const PREFIX: Command = CommandBuilder::new()
    .name("prefix")
    .description("Add prefix to each line.")
    .options(&[DELETE.arg, HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .positionals(&[VALUE.arg])
    .run(run)
    .done();

fn run(mut args: Args) -> anyhow::Result<()> {
    let delete = args.get(&DELETE);
    let prefix = args.get_owned(&VALUE);
    let prefix = prefix.as_slice();
    let context = Context::new(&args);

    let mut reader = context.line_reader();
    let mut writer = context.writer();

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
