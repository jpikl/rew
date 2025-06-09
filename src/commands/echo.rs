use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::common_options;
use crate::global::GENERATOR_COMMANDS;
use crate::run::ContextExt;
use std::ffi::OsString;

pub const ARG: Pos<OsString> = PosBuilder::new()
    .name("ARG")
    .description("Arguments to be printed on each line.")
    .description_ex(&["Their values will be separated by a space."])
    .multiple()
    .done();

pub const ECHO: Command = CommandBuilder::new()
    .name("echo")
    .description("Repeatedly output string as a line.")
    .group(&GENERATOR_COMMANDS)
    .options(common_options![])
    .positionals(&[&ARG.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut writer = ctx.writer();
    let mut buf: Vec<u8> = Vec::new();

    for (i, arg) in ctx.args.iter_ref(&ARG).enumerate() {
        if i > 0 {
            buf.push(b' ');
        }
        buf.extend_from_slice(arg.as_encoded_bytes());
    }

    loop {
        writer.write_line(buf.as_slice())?;
    }
}
