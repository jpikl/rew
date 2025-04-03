mod cli;
mod commands;
mod global;
mod io;
mod run;

use cli::Command;
use cli::CommandBuilder;
use cli::HELP;
use cli::VERSION;
use commands::cat::CAT;
use global::BUF_MODE;
use global::BUF_SIZE;
use global::NULL;

const REW: Command = CommandBuilder::new()
    .name(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .version(env!("CARGO_PKG_VERSION"))
    .options(&[HELP.arg, VERSION.arg, NULL.arg, BUF_SIZE.arg, BUF_MODE.arg])
    .commands(&[CAT])
    .done();

fn main() -> anyhow::Result<()> {
    let (command, args) = REW.parse_args()?;
    command.run(args)
}
