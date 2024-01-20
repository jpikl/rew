mod args;
mod command;
mod commands;
mod io;

use anyhow::Error;
use anyhow::Result;
use args::GlobalArgs;
use clap::command;
use clap::Args;
use clap::Command;
use owo_colors::OwoColorize;
use std::io::Write;

fn main() {
    handle_error(run().or_else(ignore_broken_pipe));
}

fn run() -> Result<()> {
    let commands = commands::get_meta();
    let app = build_app(&commands);

    if let Some((name, matches)) = app.get_matches().subcommand() {
        for command in commands {
            if name == command.name {
                return (command.run)(matches);
            }
        }
    }

    unreachable!("clap should handle missing or invalid command");
}

fn build_app(commands: &Vec<&'static command::Meta>) -> Command {
    let mut app = command!("rew").subcommand_required(true);

    for command in commands {
        app = app.subcommand((command.build)());
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

fn handle_error(result: Result<()>) {
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

fn ignore_broken_pipe(error: Error) -> Result<()> {
    for cause in error.chain() {
        if let Some(io_error) = cause.downcast_ref::<std::io::Error>() {
            if io_error.kind() == std::io::ErrorKind::BrokenPipe {
                return Ok(());
            }
        }
    }
    Err(error)
}
