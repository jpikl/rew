use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::global::BUF_MODE;
use crate::global::BUF_SIZE;
use crate::global::GENERATOR_COMMANDS;
use crate::global::HELP;
use crate::global::NULL;
use crate::run::ContextExt;
use std::ffi::OsString;

pub const VALUES: Pos<OsString> = PosBuilder::new("value")
    .description("Values to output.")
    .name("VALUE")
    .multiple()
    .done();

pub const STREAM: Command = CommandBuilder::new()
    .name("stream")
    .description("Output arguments as lines.")
    .group(&GENERATOR_COMMANDS)
    .options(&[HELP.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .positionals(&[VALUES.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut writer = ctx.writer();

    for value in ctx.args.iter_ref(&VALUES) {
        writer.write_line(value.as_encoded_bytes())?;
    }

    Ok(())
}
