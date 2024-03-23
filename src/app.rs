use crate::args::get_bin_name;
use crate::args::GlobalArgs;
use crate::colors::BOLD;
use crate::colors::RESET;
use crate::command;
use clap::command;
use clap::crate_name;
use clap::crate_version;
use clap::Args;
use clap::Command;
use std::env;

const REFERENCE_URL: &str = "https://jpikl.github.io/rew/reference";

pub fn build(metas: &[&'static command::Meta]) -> Command {
    let mut app = command!()
        .version(get_version())
        .after_help(get_after_help(None))
        .subcommand_required(true);

    for meta in metas {
        let command = meta.build().after_help(get_after_help(Some(meta.name)));
        app = app.subcommand(command);
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

fn get_version() -> String {
    let version = crate_version!();
    let hash = env!("BUILD_COMMIT_HASH");
    let date = env!("BUILD_COMMIT_DATE");

    format!("{version} ({hash} {date})")
}

fn get_after_help(cmd: Option<&str>) -> String {
    let app = crate_name!();
    let file = if let Some(cmd) = cmd {
        format!("{app}-{cmd}.html")
    } else {
        format!("{app}.html")
    };
    format!("Visit {BOLD}{REFERENCE_URL}/{file}.html{RESET} for a complete reference and examples.")
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
        format!("{BOLD}{prefix}{RESET} (spawned by '{BOLD}{spawned_by}{RESET}')")
    } else {
        format!("{BOLD}{prefix}{RESET}")
    }
}
