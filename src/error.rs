use crate::app::get_prefix;
use crate::args::ENV_SPAWNED_BY;
use anstream::eprintln;
use clap::error::ErrorKind;
use clap::Command;
use color_print::cstr;
use std::env;

const ARGS_ERROR: &str = cstr!("<red,bold>invalid usage:");
const RUN_ERROR: &str = cstr!("<red,bold>error:");

pub struct Reporter {
    spawned_by: Option<String>,
    prefix: String,
}

impl Reporter {
    pub fn new(app: &Command) -> Self {
        let spawned_by = env::var(ENV_SPAWNED_BY).ok();
        let prefix = get_prefix(app, spawned_by.as_deref());
        Self { spawned_by, prefix }
    }

    pub fn print_args_error(&self, error: &clap::Error) {
        match error.kind() {
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => error.exit(),
            _ => {}
        }

        if self.spawned_by.is_some() {
            // Be brief when spawned by another process
            let message = error.kind().as_str().unwrap_or("unknown error");
            eprintln!("{}: {ARGS_ERROR} {message}", self.prefix);
        } else {
            let message = error.render().ansi().to_string();
            let message = message.replacen("error:", ARGS_ERROR, 1);
            eprint!("{}: {message}", self.prefix);
        };
    }

    pub fn print_run_error(&self, error: &anyhow::Error) {
        eprintln!("{}: {RUN_ERROR} {error}", self.prefix);

        for cause in error.chain().skip(1) {
            eprintln!("{}: └─> {cause}", self.prefix);
        }
    }
}
