use crate::cli::Cli;
use crate::input::{ArgsInput, Input, StdinInput};
use crate::pattern::{Lexer, Parser, Pattern};
use crate::state::{RegexTarget, State};
use std::io::{self, Write};
use std::process;
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

    let mut parser = Parser::new(lexer);
    let pattern = match parser.parse_items() {
        Ok(items) => Pattern::new(items),
        Err(error) => {
            writeln!(&mut stderr, "error: {}", error.kind)?;

            if !raw_pattern.is_empty() {
                writeln!(&mut stderr)?;
                Pattern::render(&mut stderr, raw_pattern)?;

                let spaces_count = raw_pattern[..error.start].chars().count();
                let markers_count = raw_pattern[error.start..error.end].chars().count().max(1);

                write!(&mut stderr, "\n{}", " ".repeat(spaces_count))?;
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(&mut stderr, "{}", "^".repeat(markers_count))?;

                stderr.reset()?;
                writeln!(&mut stderr)?;
            }

            process::exit(2);
        }
    };

    let mut state = State::new();
    state.set_local_counter_enabled(pattern.uses_local_counter());
    state.set_global_counter_enabled(pattern.uses_global_counter());

    if pattern.uses_regex_captures() {
        if let Some(regex) = cli.regex_filename() {
            state.set_regex(Some(regex));
            state.set_regex_target(RegexTarget::Filename);
        } else if let Some(regex) = cli.regex_path() {
            state.set_regex(Some(regex));
            state.set_regex_target(RegexTarget::Path);
        }
    }

    let mut input: Box<dyn Input> = if let Some(files) = cli.paths() {
        Box::new(ArgsInput::new(files))
    } else {
        Box::new(StdinInput::new(&mut stdin, cli.read_nul()))
    };

    let delimiter = if cli.print_raw() {
        None
    } else if cli.print_nul() {
        Some('\0')
    } else {
        Some('\n')
    };

    while let Some(src_path) = input.next()? {
        // TODO handle error
        let eval_context = state.get_eval_context(src_path);
        let dst_path = pattern.eval(&eval_context).unwrap();

        if let Some(delimiter_value) = delimiter {
            write!(&mut stdout, "{}{}", dst_path, delimiter_value)?;
        } else {
            write!(&mut stdout, "{}", dst_path)?;
        }
    }

    Ok(())
}
