use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::cli::QUOTE_END;
use crate::cli::QUOTE_START;
use crate::common_options;
use crate::format::Formatter;
use crate::global::GENERATOR_COMMANDS;
use crate::run::ContextExt;
use rand::Rng;

const FROM: Pos<i128> = PosBuilder::new()
    .name("FROM")
    .description("Lower inclusive bound of the generated range.")
    .default("0")
    .negative()
    .done();

const TO: Pos<i128> = PosBuilder::new()
    .name("TO")
    .description("Upper inclusive bound of the generated range.")
    .negative()
    .done();

pub const RAND: Command = CommandBuilder::new()
    .name("rand")
    .description("Generate random numbers as lines.")
    .group(&GENERATOR_COMMANDS)
    .options(common_options![])
    .positionals(&[&FROM.arg, &TO.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let from = ctx.args.get(&FROM);
    let to = ctx.args.get_opt(&TO).unwrap_or(i128::MAX);

    if from > to {
        return Err(ErrorKind::InvalidUsage(format!(
            "{QUOTE_START}{}{QUOTE_END} value {QUOTE_START}{from}{QUOTE_END} cannot be greater than {QUOTE_START}{}{QUOTE_END} value {QUOTE_START}{to}{QUOTE_END}",
            FROM.arg, TO.arg
        )));
    }

    let mut writer = ctx.writer();
    let mut rng = rand::rng();
    let mut fmt = Formatter::with_buf_size(64); // Enough for digits of i128

    loop {
        let num = rng.random_range(from..=to);
        writer.write_line(fmt.format_int(num))?
    }
}
