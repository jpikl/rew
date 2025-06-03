use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::common_options;
use crate::global::GENERATOR_COMMANDS;
use crate::run::ContextExt;
use uuid::Uuid;

pub const UUID: Command = CommandBuilder::new()
    .name("uuid")
    .description("Generate stream of UUID v4 as lines.")
    .group(&GENERATOR_COMMANDS)
    .options(common_options![])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let mut writer = ctx.writer();
    let mut buf = [0u8; 36];

    loop {
        Uuid::new_v4().as_hyphenated().encode_lower(&mut buf);
        writer.write_line(buf.as_slice())?
    }
}
