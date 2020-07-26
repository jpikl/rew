use crate::cli::Cli;
use crate::pattern::{eval, Pattern};
use std::io::Stdin;
use std::{env, io, process};
use structopt::StructOpt;
use termcolor::{ColorChoice, StandardStream};

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
mod regex;
mod utils;

const ERR_IO: i32 = 1 << 1;
const ERR_PARSE: i32 = 1 << 2;
const ERR_EVAL: i32 = 1 << 3;
const ERR_REGEX: i32 = 1 << 4;

fn main() {
    // Explicit variable type, because IDE is unable to detect it.
    let cli: Cli = Cli::from_args();

    let color = match cli.color {
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
    let mut stdout = StandardStream::stdout(color);
    let mut stderr = StandardStream::stderr(color);

    if let Some(error) = run(&cli, &mut stdin, &mut stdout, &mut stderr).err() {
        output::Errors::new(&mut stderr)
            .write(&error)
            .expect("Failed to write to stderr!");
        process::exit(ERR_IO);
    }
}

fn run(
    cli: &Cli,
    stdin: &mut Stdin,
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
) -> Result<(), io::Error> {
    let mut output_errors = output::Errors::new(stderr);

    let pattern = match Pattern::parse(&cli.pattern, cli.escape) {
        Ok(pattern) => pattern,
        Err(error) => {
            output_errors.write_with_highlight(&error, &cli.pattern)?;
            process::exit(ERR_PARSE);
        }
    };

    if cli.explain {
        pattern.explain(stdout)?;
        process::exit(0);
    }

    let global_counter_used = pattern.uses_global_counter();
    let local_counter_used = pattern.uses_local_counter();
    let regex_captures_used = pattern.uses_regex_captures();

    let mut global_counter_generator =
        counter::GlobalGenerator::new(cli.gc_init.unwrap_or(1), cli.gc_step.unwrap_or(1));

    let mut local_counter_generator =
        counter::LocalGenerator::new(cli.lc_init.unwrap_or(1), cli.lc_step.unwrap_or(1));

    let regex_solver = if let Some(regex) = &cli.regex {
        regex::Solver::FileName(regex)
    } else if let Some(regex) = &cli.regex_full {
        regex::Solver::Path(regex)
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
        input::Paths::from_stdin(stdin, input_delimiter)
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

    let current_dir_buf = env::current_dir()?;
    let current_dir = current_dir_buf.as_path();
    let mut output_paths = output::Paths::new(stdout, output_delimiter);
    let mut exit_code = 0;

    while let Some(path) = input_paths.next()? {
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
                    if cli.fail_at_end {
                        exit_code |= ERR_REGEX;
                        continue;
                    } else {
                        process::exit(ERR_REGEX);
                    }
                }
            }
        } else {
            None
        };

        let context = eval::Context {
            path,
            current_dir,
            global_counter,
            local_counter,
            regex_captures,
        };

        let output_path = match pattern.eval(&context) {
            Ok(path) => path,
            Err(error) => {
                output_errors.write_with_highlight(&error, &cli.pattern)?;
                if cli.fail_at_end {
                    exit_code |= ERR_EVAL;
                    continue;
                } else {
                    process::exit(ERR_EVAL);
                }
            }
        };

        output_paths.write(&output_path)?;
    }

    process::exit(exit_code);
}
