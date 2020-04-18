use crate::cli::Cli;
use crate::input::{ArgsInput, Input, StdinInput};
use crate::pattern::{Lexer, Pattern};
use crate::state::State;
use std::io::{self, Write};
use std::{cmp, process};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod cli;
mod input;
mod pattern;
mod state;

fn main() -> Result<(), io::Error> {
    let cli = Cli::new();

    let color_choice = match cli.color() {
        Some(ColorChoice::Auto) | None => {
            if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        }
        Some(other) => other,
    };

    let mut stdin = io::stdin();
    let mut stdout = StandardStream::stdout(color_choice);
    let mut stderr = StandardStream::stderr(color_choice);

    let raw_pattern = cli.pattern();
    let mut lexer = Lexer::from(raw_pattern);

    if let Some(escape) = cli.escape() {
        lexer.set_escape(escape);
    }

    match Pattern::parse_tokens(lexer) {
        Ok(pattern) => {
            let mut state = State::new();
            state.set_local_counter_enabled(pattern.uses_local_counter());
            state.set_global_counter_enabled(pattern.uses_global_counter());

            if pattern.uses_regex_captures() {
                state.set_regex(cli.regex());

                if let Some(regex_target) = cli.regex_target() {
                    state.set_regex_target(regex_target);
                }
            }

            let mut input: Box<dyn Input> = if let Some(files) = cli.paths() {
                Box::new(ArgsInput::new(files))
            } else {
                Box::new(StdinInput::new(&mut stdin, cli.zero_terminated_stdin()))
            };

            while let Some(src_path) = input.next()? {
                // TODO handle error
                let eval_context = state.get_eval_context(src_path);
                let dst_path = pattern.eval(&eval_context).unwrap();
                writeln!(&mut stdout, "{}", dst_path)?;
            }

            Ok(())
        }
        Err(error) => {
            writeln!(&mut stderr, "error: {}", error.kind)?;

            if !raw_pattern.is_empty() {
                writeln!(&mut stderr)?;
                Pattern::render(&mut stderr, raw_pattern)?;

                write!(&mut stderr, "\n{}", " ".repeat(error.start))?;
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(
                    &mut stderr,
                    "{}",
                    "^".repeat(cmp::max(1, error.end - error.start))
                )?;

                stderr.reset()?;
                writeln!(&mut stderr)?;
            }

            process::exit(2);
        }
    }
}
