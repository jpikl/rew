use crate::cli::Command;
use crate::cli::CommandBuilder;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Opt;
use crate::cli::OptBuilder;
use crate::cli::Pos;
use crate::cli::PosBuilder;
use crate::cli::SimpleError;
use crate::common_options;
use crate::format::Formatter;
use crate::global::GENERATOR_COMMANDS;
use crate::run::ContextExt;

const FIRST: Pos<i128> = PosBuilder::new()
    .name("FIRST")
    .description("First number in the sequence.")
    .default("0")
    .done();

const LAST: Pos<i128> = PosBuilder::new()
    .name("LAST")
    .description("Last number in the sequence.")
    .done();

const INC: Opt<i128> = OptBuilder::new()
    .short('i')
    .long("increment")
    .description("Increment between numbers.")
    .done();

pub const SEQ: Command = CommandBuilder::new()
    .name("seq")
    .description("Generate number sequence as lines.")
    .group(&GENERATOR_COMMANDS)
    .options(common_options![&INC.arg])
    .positionals(&[&FIRST.arg, &LAST.arg])
    .run(run)
    .done();

fn run(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let first = ctx.args.get(&FIRST);
    let last = ctx.args.get_opt(&LAST);
    let inc = ctx.args.get_opt(&INC);

    let mut value = first;
    let mut writer = ctx.writer();
    let mut fmt = Formatter::with_buf_size(64); // Enough for digits of i128

    match last {
        Some(last) if first < last => {
            let inc = inc.unwrap_or(1);
            while value <= last {
                writer.write_line(fmt.format_int(value))?;
                value += inc;
            }
        }
        Some(last) if first > last => {
            let inc = inc.unwrap_or(-1);
            while value >= last {
                writer.write_line(fmt.format_int(value))?;
                value += inc;
            }
        }
        Some(_) => {
            // first == last
            writer.write_line(fmt.format_int(value))?;
        }
        None => {
            let inc = inc.unwrap_or(1);
            loop {
                writer.write_line(fmt.format_int(value))?;
                if let Some(new_value) = value.checked_add(inc) {
                    value = new_value;
                } else {
                    return Err(SimpleError::new("Number sequence reached integer limit").into());
                }
            }
        }
    }

    Ok(())
}
