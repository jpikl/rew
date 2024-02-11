use crate::args::GlobalArgs;
use crate::command;
use anyhow::Error;
use anyhow::Result;
use clap::command;
use clap::Args;
use clap::Command;
use owo_colors::OwoColorize;
use std::io::Write;

pub fn build_app(commands: &[&'static command::Meta]) -> Command {
    let mut app = command!("rew").subcommand_required(true);

    for command in commands {
        app = app.subcommand((command.build)());
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

pub fn handle_error(result: Result<()>) {
    if let Err(error) = result {
        report_error(&error, &mut anstream::stderr()).expect("Failed to write error to stderr!");
        std::process::exit(1);
    }
}

fn report_error(error: &Error, stderr: &mut impl Write) -> std::io::Result<()> {
    writeln!(stderr, "{}: {}", "error".red().bold(), error)?;

    for cause in error.chain().skip(1) {
        writeln!(stderr, "{}: {}", "cause".red(), cause)?;
    }

    Ok(())
}
