use crate::cli::Cli;
use crate::pattern::{eval, Pattern};
use std::{io, process};
use structopt::StructOpt;
use termcolor::{ColorChoice, StandardStream};

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
mod regex;
mod utils;

fn main() -> Result<(), io::Error> {
    // Explicit variable type, because IDE is unable to detect it.
    let cli: Cli = Cli::from_args();

    let output_colors = match cli.color {
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
    let mut stdout = StandardStream::stdout(output_colors);
    let mut stderr = StandardStream::stderr(output_colors);
    let mut output_errors = output::Errors::new(&mut stderr);

    let pattern = match Pattern::parse(&cli.pattern, cli.escape) {
        Ok(pattern) => pattern,
        Err(error) => {
            output_errors.write_with_highlight(&error, &cli.pattern)?;
            process::exit(2);
        }
    };

    if cli.explain {
        return pattern.explain(&mut stdout);
    }

    let global_counter_used = pattern.uses_global_counter();
    let local_counter_used = pattern.uses_local_counter();
    let regex_captures_used = pattern.uses_regex_captures();

    let mut global_counter_generator =
        counter::GlobalGenerator::new(cli.gc_init.unwrap_or(1), cli.gc_step.unwrap_or(1));

    let mut local_counter_generator =
        counter::LocalGenerator::new(cli.lc_init.unwrap_or(1), cli.lc_step.unwrap_or(1));

    let regex_solver = if let Some(regex) = &cli.regex {
        regex::Solver::Filename(regex)
    } else if let Some(regex) = &cli.regex_full {
        regex::Solver::FullPath(regex)
    } else {
        regex::Solver::None
    };

    let mut input_paths = if cli.paths.is_empty() {
        let input_delimiter = if cli.read_raw {
            None
        } else if cli.read_nul {
            Some(0)
        } else {
            Some(b'\n')
        };
        input::Paths::from_stdin(&mut stdin, input_delimiter)
    } else {
        input::Paths::from_args(cli.paths.as_slice())
    };

    let output_delimiter = if cli.print_raw {
        None
    } else if cli.print_nul {
        Some('\0')
    } else {
        Some('\n')
    };

    let mut output_paths = output::Paths::new(&mut stdout, output_delimiter);

    loop {
        match input_paths.next() {
            Ok(Some(path)) => {
                let global_counter = if global_counter_used {
                    global_counter_generator.next()
                } else {
                    0
                };

                let local_counter = if local_counter_used {
                    local_counter_generator.next(path)
                } else {
                    0
                };

                let regex_captures = if regex_captures_used {
                    match regex_solver.eval(path) {
                        Ok(captures) => captures,
                        Err(error) => {
                            output_errors.write(&error)?;
                            process::exit(4)
                        }
                    }
                } else {
                    None
                };

                let context = eval::Context {
                    path,
                    global_counter,
                    local_counter,
                    regex_captures,
                };

                let output_path = match pattern.eval(&context) {
                    Ok(path) => path,
                    Err(error) => {
                        output_errors.write_with_highlight(&error, &cli.pattern)?;
                        process::exit(3);
                    }
                };

                output_paths.write(&output_path)?;
            }

            Ok(None) => break,
            Err(error) => {
                output_errors.write(&error)?;
                process::exit(5)
            }
        }
    }

    Ok(())
}
