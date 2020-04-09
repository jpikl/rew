use crate::cli::Cli;
use crate::input::{ArgsInput, Input, StdinInput};
use crate::pattern::{EvalContext, Pattern};
use std::io::{self, Write};
use std::{cmp, process};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

mod cli;
mod input;
mod pattern;

fn main() -> Result<(), io::Error> {
    let cli = Cli::new();

    let raw_pattern = cli.pattern();
    let color_choice = cli.color();

    let mut stdin = io::stdin();
    let mut stdout = StandardStream::stdout(color_choice);
    let mut stderr = StandardStream::stderr(color_choice);

    match Pattern::parse(raw_pattern) {
        Ok(pattern) => {
            let mut input: Box<dyn Input> = if let Some(files) = cli.files() {
                Box::new(ArgsInput::new(files))
            } else {
                Box::new(StdinInput::new(&mut stdin, cli.zero_terminated()))
            };
            while let Some(src_path) = input.next()? {
                let dst_path = pattern
                    .eval(&mut EvalContext {
                        path: src_path,
                        global_counter: 0,
                        local_counter: 0,
                        capture_groups: Vec::new(),
                    })
                    .unwrap();
                writeln!(&mut stdout, "{}", dst_path)?;
            }
            Ok(())
        }
        Err(error) => {
            writeln!(&mut stderr, "{}", error.typ)?;
            if !raw_pattern.is_empty() {
                writeln!(&mut stderr, "\n")?;
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
