use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Fmt;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::common_options;
use crate::global::GENERATOR_COMMANDS;
use crate::run::ContextExt;
use rand::Rng;

const FROM: Pos<u64> = PosBuilder::new("from")
    .name("FROM")
    .description("Lower inclusive bound of the generated range.")
    .default("0")
    .done();

const TO: Pos<u64> = PosBuilder::new("to")
    .name("TO")
    .description("Upper inclusive bound of the generated range.")
    .done();

pub const RAND: Command = CommandBuilder::new()
    .name("rand")
    .description("Generate stream of random numbers as lines.")
    .group(&GENERATOR_COMMANDS)
    .options(common_options![])
    .positionals(&[FROM.arg, TO.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let from = ctx.args.get(&FROM);
    let to = ctx.args.get_opt(&TO).unwrap_or(u64::MAX);

    if from > to {
        return Err(ErrorKind::InvalidUsage(format!(
            "{} value {} cannot be greater than {} value {}",
            Fmt::quote(FROM.arg),
            Fmt::quote(from),
            Fmt::quote(TO.arg),
            Fmt::quote(to)
        )));
    }

    let mut writer = ctx.writer();
    let mut rng = rand::rng();
    let mut buf: Vec<u8> = Vec::with_capacity(64);

    loop {
        let num = rng.random_range(from..=to);
        buf.clear();
        itoap::write_to_vec(&mut buf, num);
        writer.write_line(buf.as_slice())?
    }
}
