use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::Opt;
use crate::cli::OptBuilder;
use crate::common_options;
use crate::global::MAP_COMMANDS;
use crate::run::ContextExt;
use bstr::BString;
use bstr::ByteSlice;

const DOUBLE: Flag = FlagBuilder::new()
    .short('d')
    .long("double")
    .description("Use double quotes instead of single.")
    .done();

const ESCAPE: Opt<BString> = OptBuilder::new()
    .short('e')
    .long("escape")
    .value_name("CHAR")
    .description("Character to use when escaping inner quotes.")
    .default("\\")
    .done();

const NO_ESCAPE: Flag = FlagBuilder::new()
    .short('E')
    .long("no-escape")
    .description("Do not escape inner quotes.")
    .done();

pub const QUOTE: Command = CommandBuilder::new()
    .name("quote")
    .description("Put each line into quotes.")
    .description_ex(&["Also escapes quote characters inside."])
    .group(&MAP_COMMANDS)
    .options(common_options![&DOUBLE.arg, &ESCAPE.arg, &NO_ESCAPE.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let double = ctx.args.get(&DOUBLE);
    let escape = ctx.args.get_ref(&ESCAPE);
    let no_escape = ctx.args.get(&NO_ESCAPE) || escape.is_empty();
    let quote = if double { b'"' } else { b'\'' };

    let mut reader = ctx.line_reader();
    let mut writer = ctx.writer();

    while let Some(line) = reader.read_line()? {
        writer.write(&[quote])?;

        if no_escape {
            writer.write(line)?;
        } else {
            let mut start = 0;

            while let Some(pos) = line[start..].find_byte(quote) {
                writer.write(&line[start..][..pos])?;
                writer.write(&escape)?;
                writer.write(&[quote])?;
                start += pos + 1;
            }

            writer.write(&line[start..])?;
        }

        writer.write(&[quote])?;
        writer.write_separator()?;
    }

    Ok(())
}
