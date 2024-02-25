use crate::app::get_prefix;
use crate::args::ENV_SPAWNED_BY;
use crate::colors::write_help;
use crate::colors::BOLD_RED;
use crate::colors::RESET;
use anstream::eprintln;
use anstream::stdout;
use anyhow::Context;
use clap::Command;
use std::env;

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

    pub fn print_help(&self, error: &clap::Error) {
        let mut stdout = stdout().lock();
        let help = error.render();

        if let Err(error) = write_help(&mut stdout, &help).context("could not write to stdout") {
            self.print_error(&error);
        }
    }

    pub fn print_invalid_usage(&self, error: &clap::Error) {
        let err_prefix = "invalid usage";

        if self.spawned_by.is_some() {
            // Be brief when spawned by another process
            let message = error.kind().as_str().unwrap_or("unknown error");
            eprintln!("{}: {BOLD_RED}{err_prefix}:{RESET} {message}", self.prefix);
        } else {
            let message = error.render().ansi().to_string();
            let message = message.replacen("error", err_prefix, 1);
            eprint!("{}: {message}", self.prefix);
        };
    }

    pub fn print_error(&self, error: &anyhow::Error) {
        eprintln!("{}: {BOLD_RED}error:{RESET} {error}", self.prefix);

        for cause in error.chain().skip(1) {
            eprintln!("{}: └─> {cause}", self.prefix);
        }
    }
}
