use crate::args::get_bin_name;
use crate::args::GlobalArgs;
use crate::command;
use clap::command;
use clap::Args;
use clap::Command;
use color_print::cformat;
use std::env;

pub fn build(commands: &[&'static command::Meta]) -> Command {
    let mut app = command!().subcommand_required(true);

    for command in commands {
        app = app.subcommand((command.build)());
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

pub fn get_prefix(app: &Command, spawned_by: Option<&str>) -> String {
    let bin_name = get_bin_name();

    let prefix = app
        .clone()
        .ignore_errors(true)
        .try_get_matches()
        .ok()
        .and_then(|matches| matches.subcommand_name().map(ToString::to_string))
        .map(|cmd_name| format!("{} {cmd_name}", &bin_name))
        .unwrap_or(bin_name);

    if let Some(spawned_by) = spawned_by {
        cformat!("<bold>{prefix}</> (spawned by '<bold>{spawned_by}</>')")
    } else {
        cformat!("<bold>{prefix}</>")
    }
}
