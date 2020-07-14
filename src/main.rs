use crate::cli::Cli;
use crate::input::Input;
use crate::pattern::{EvalContext, Lexer, Parser, Pattern};
use std::collections::HashMap;
use std::io::{self, Write};
use std::process;
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod cli;
mod input;
mod pattern;

fn main() -> Result<(), io::Error> {
    // Explicit variable type, because IDE is unable to detect it.
    let cli: Cli = Cli::from_args();

    let color_choice = match cli.color {
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

    let raw_pattern = cli.pattern.as_str();
    let mut lexer = Lexer::from(raw_pattern);

    if let Some(escape) = cli.escape {
        lexer.set_escape(escape);
    }

    let pattern = match Parser::new(lexer).parse_items() {
        Ok(items) => Pattern::new(items),
        Err(error) => {
            writeln!(&mut stderr, "error: {}", error.kind)?;

            if !raw_pattern.is_empty() {
                writeln!(&mut stderr)?;
                Pattern::render(&mut stderr, raw_pattern)?;

                let spaces_count = raw_pattern[..error.range.start].chars().count();
                let markers_count = raw_pattern[error.range].chars().count().max(1);

                write!(&mut stderr, "\n{}", " ".repeat(spaces_count))?;
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(&mut stderr, "{}", "^".repeat(markers_count))?;

                stderr.reset()?;
                writeln!(&mut stderr)?;
            }

            process::exit(2);
        }
    };

    let mut input = if cli.paths.is_empty() {
        let delimiter = if cli.read_nul { 0 } else { b'\n' };
        Input::from_stdin(&mut stdin, delimiter)
    } else {
        Input::from_args(cli.paths.as_slice())
    };

    let delimiter = if cli.print_raw {
        None
    } else if cli.print_nul {
        Some('\0')
    } else {
        Some('\n')
    };

    let global_counter_used = pattern.uses_global_counter();
    let local_counter_used = pattern.uses_local_counter();
    let regex_captures_used = pattern.uses_regex_captures();

    let mut global_counter = 0; // TODO init value from cli option
    let mut directory_local_counters = HashMap::new();

    while let Some(src_path) = input.next()? {
        if global_counter_used {
            global_counter += 1; // TODO increment from cli option
        }

        let local_counter = if local_counter_used {
            if let Some(directory) = src_path.parent() {
                let directory_buf = directory.to_path_buf();
                if let Some(counter) = directory_local_counters.get_mut(&directory_buf) {
                    *counter += 1; // TODO increment from cli option
                    *counter
                } else {
                    // TODO init value from cli option
                    directory_local_counters.insert(directory_buf, 1);
                    1
                }
            } else {
                0
            }
        } else {
            0
        };

        let regex_captures = if regex_captures_used {
            if let Some(regex) = &cli.regex {
                src_path
                    .file_name()
                    .map(|file_name| regex.captures(file_name.to_str().unwrap())) // TODO handle utf error
                    .flatten()
            } else if let Some(regex) = &cli.regex_full {
                regex.captures(src_path.to_str().unwrap()) // TODO handle utf error
            } else {
                None
            }
        } else {
            None
        };

        let eval_context = EvalContext {
            path: src_path,
            global_counter,
            local_counter,
            regex_captures,
        };

        let dst_path = pattern.eval(&eval_context).unwrap(); // TODO handle error

        if let Some(delimiter_value) = delimiter {
            write!(&mut stdout, "{}{}", dst_path, delimiter_value)?;
        } else {
            write!(&mut stdout, "{}", dst_path)?;
        }
    }

    Ok(())
}
