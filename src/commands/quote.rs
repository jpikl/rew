use crate::cli::Args;
use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::HELP;
use crate::cli::Opt;
use crate::cli::OptBuilder;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::NULL;
use crate::run::Context;
use crate::utils::into_bytes;
use memchr::memchr;
use std::ffi::OsString;

const DOUBLE: Flag = FlagBuilder::new("double")
    .short('d')
    .long("double")
    .description("Use double quotes instead of single.")
    .done();

const ESCAPE: Opt<OsString> = OptBuilder::new("escape")
    .short('e')
    .long("escape")
    .value_name("CHAR")
    .description("Character to use when escaping inner quotes.")
    .default("\\")
    .done();

const NO_ESCAPE: Flag = FlagBuilder::new("no-escape")
    .short('E')
    .long("no-escape")
    .description("Do not escape inner quotes.")
    .done();

pub const QUOTE: Command = CommandBuilder::new()
    .name("quote")
    .description("Put each input line in quotes.")
    .description_ex(&["Also escapes quote characters inside."])
    .options(&[
        DOUBLE.arg,
        ESCAPE.arg,
        NO_ESCAPE.arg,
        HELP.arg,
        NULL.arg,
        BUF_SIZE.arg,
        BUF_MODE.arg,
    ])
    .run(run)
    .done();

fn run(mut args: Args) -> anyhow::Result<()> {
    let double = args.get(&DOUBLE);
    let escape = into_bytes(args.get_owned(&ESCAPE));
    let no_escape = args.get(&NO_ESCAPE) || escape.is_empty();
    let quote = if double { b'"' } else { b'\'' };

    let context = Context::new(&args);
    let mut reader = context.line_reader();
    let mut writer = context.writer();

    while let Some(line) = reader.read_line()? {
        writer.write(&[quote])?;

        if no_escape {
            writer.write(line)?;
        } else {
            let mut start = 0;

            while let Some(pos) = memchr(quote, &line[start..]) {
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
