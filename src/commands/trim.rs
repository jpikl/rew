use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::common_options;
use crate::global::MAP_COMMANDS;
use crate::run::ContextExt;
use bstr::ByteSlice;

const START: Flag = FlagBuilder::new("start")
    .short('s')
    .long("start")
    .description("Trim start of each line.")
    .done();

const END: Flag = FlagBuilder::new("end")
    .short('e')
    .long("end")
    .description("Trim end of each line.")
    .done();

pub const TRIM: Command = CommandBuilder::new()
    .name("trim")
    .description("Trim whitespaces from each line.")
    .group(&MAP_COMMANDS)
    .options(common_options![&START.arg, &END.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let start = ctx.args.get(&START);
    let end = ctx.args.get(&END);

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        let result = match (start, end) {
            (true, true) | (false, false) => line.trim(),
            (true, false) => line.trim_start(),
            (false, true) => line.trim_end(),
        };
        writer.write_line(result)?;
    }

    Ok(())
}
