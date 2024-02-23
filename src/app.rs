use crate::args::get_bin_name;
use crate::args::GlobalArgs;
use crate::command;
use clap::command;
use clap::Args;
use clap::Command;
use color_print::cformat;
use std::env;

const REFERENCE_URL: &str = "https://jpikl.github.io/rew/reference";

pub fn build(metas: &[&'static command::Meta]) -> Command {
    let mut app = command!().subcommand_required(true);
    let app_name = app.get_name().to_string();

    app = app.after_help(get_after_help(&app_name, None));

    for meta in metas {
        let command = (meta.build)().after_help(get_after_help(&app_name, Some(meta.name)));
        app = app.subcommand(command);
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

fn get_after_help(app: &str, cmd: Option<&str>) -> String {
    let file_stem = if let Some(cmd) = cmd {
        format!("{app}-{cmd}")
    } else {
        app.to_owned()
    };
    cformat!(
        "Visit <bold>{REFERENCE_URL}/{file_stem}.html</> for a complete reference and examples."
    )
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
