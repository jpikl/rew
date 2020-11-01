use crate::cli::Cli;
use crate::output::write_pattern_error;
use crate::pattern::{eval, help, Pattern};
use common::input::Delimiter as InputDelimiter;
use common::output::write_error;
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};
use std::env;

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
mod regex;
#[cfg(test)]
mod testing;
mod utils;

const EXIT_CODE_PATTERN_PARSE_ERROR: i32 = 3;
const EXIT_CODE_PATTERN_EVAL_ERROR: i32 = 4;
const EXIT_CODE_REGEX_EVAL_ERROR: i32 = 5;

fn main() {
    exec_run(run);
}

fn run(cli: &Cli, io: &Io) -> Result {
    if cli.help_pattern {
        help::write_pattern_help(&mut io.stdout())?;
        return Ok(EXIT_CODE_OK);
    }

    if cli.help_vars {
        help::write_variables_help(&mut io.stdout())?;
        return Ok(EXIT_CODE_OK);
    }

    if cli.help_filters {
        help::write_filters_help(&mut io.stdout())?;
        return Ok(EXIT_CODE_OK);
    }

    let pattern = match Pattern::parse(cli.pattern(), cli.escape) {
        Ok(pattern) => pattern,
        Err(error) => {
            write_pattern_error(&mut io.stderr(), &error, cli.pattern())?;
            return Ok(EXIT_CODE_PATTERN_PARSE_ERROR);
        }
    };

    if cli.explain {
        pattern.explain(&mut io.stdout())?;
        return Ok(EXIT_CODE_OK);
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
            InputDelimiter::None
        } else if cli.read_nul {
            InputDelimiter::Nul
        } else {
            InputDelimiter::Newline
        };
        input::Paths::from_stdin(io.stdin(), input_delimiter)
    } else {
        input::Paths::from_args(cli.paths.as_slice())
    };

    let output_path_mode = if cli.pretty {
        output::PathMode::Pretty
    } else {
        let output_delimiter = if cli.print_raw {
            None
        } else if cli.print_nul {
            Some('\0')
        } else {
            Some('\n')
        };
        if cli.bulk {
            output::PathMode::Diff(output_delimiter)
        } else {
            output::PathMode::Out(output_delimiter)
        }
    };

    let mut output_paths = output::Paths::new(io.stdout(), output_path_mode);
    let mut exit_code = EXIT_CODE_OK;

    let current_dir_buf = env::current_dir()?;
    let current_dir = current_dir_buf.as_path();

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
                    write_error(&mut io.stderr(), &error)?;
                    if cli.fail_at_end {
                        exit_code = EXIT_CODE_REGEX_EVAL_ERROR;
                        continue;
                    } else {
                        return Ok(EXIT_CODE_REGEX_EVAL_ERROR);
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
                write_pattern_error(&mut io.stderr(), &error, cli.pattern())?;
                if cli.fail_at_end {
                    exit_code = EXIT_CODE_PATTERN_EVAL_ERROR;
                    continue;
                } else {
                    return Ok(EXIT_CODE_PATTERN_EVAL_ERROR);
                }
            }
        };

        output_paths.write(path, &output_path)?;
    }

    Ok(exit_code)
}
