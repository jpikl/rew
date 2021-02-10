use crate::cli::Cli;
use crate::output::write_pattern_error;
use crate::pattern::{eval, help, Pattern};
use common::help::highlight;
use common::input::Delimiter as InputDelimiter;
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};
use std::env;
use std::io::Write;

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
mod regex;
#[cfg(test)]
mod testing;
mod utils;

const EXIT_CODE_PARSE_ERROR: i32 = 3;
const EXIT_CODE_EVAL_ERROR: i32 = 4;

fn main() {
    exec_run(run);
}

fn run(cli: &Cli, io: &Io) -> Result {
    if cli.help_pattern {
        highlight(&mut io.stdout(), help::PATTERN)?;
        return Ok(EXIT_CODE_OK);
    }

    if cli.help_filters {
        highlight(&mut io.stdout(), help::FILTERS)?;
        return Ok(EXIT_CODE_OK);
    }

    let mut input_values = if cli.values.is_empty() {
        let input_delimiter = if let Some(byte) = cli.read {
            InputDelimiter::Byte(byte)
        } else if cli.read_nul {
            InputDelimiter::Byte(0)
        } else if cli.read_raw {
            InputDelimiter::None
        } else {
            InputDelimiter::Newline
        };
        input::Values::from_stdin(io.stdin(), input_delimiter)
    } else {
        input::Values::from_args(cli.values.as_slice())
    };

    let output_mode = if cli.pretty {
        output::Mode::Pretty
    } else if cli.diff {
        output::Mode::Diff
    } else if cli.no_trailing_delimiter {
        output::Mode::StandardNoTrailingDelimiter
    } else {
        output::Mode::Standard
    };

    let output_delimiter = if let Some(delimiter) = &cli.print {
        delimiter
    } else if cli.print_raw {
        ""
    } else if cli.print_nul {
        "\0"
    } else {
        "\n"
    };

    let mut output_values = output::Values::new(io.stdout(), output_mode, output_delimiter);
    let mut exit_code = EXIT_CODE_OK;

    if let Some(raw_pattern) = cli.pattern.as_ref() {
        let pattern = match Pattern::parse(raw_pattern, cli.escape) {
            Ok(pattern) => pattern,
            Err(error) => {
                write_pattern_error(&mut io.stderr(), &error, raw_pattern)?;
                return Ok(EXIT_CODE_PARSE_ERROR);
            }
        };

        if cli.explain {
            pattern.explain(&mut io.stdout())?;
            return Ok(EXIT_CODE_OK);
        }

        let global_counter_used = pattern.uses_global_counter();
        let local_counter_used = pattern.uses_local_counter();
        let regex_capture_used = pattern.uses_regex_capture();

        let global_counter_config = cli.global_counter.unwrap_or_else(counter::Config::default);
        let local_counter_config = cli.local_counter.unwrap_or_else(counter::Config::default);

        let mut global_counter_generator = counter::GlobalGenerator::from(&global_counter_config);
        let mut local_counter_generator = counter::LocalGenerator::from(&local_counter_config);

        let regex_solver = if let Some(regex) = &cli.regex {
            regex::Solver::Value(regex)
        } else if let Some(regex) = &cli.regex_filename {
            regex::Solver::FileName(regex)
        } else {
            regex::Solver::None
        };

        let working_dir = if let Some(working_dir) = &cli.working_directory {
            if working_dir.is_relative() {
                env::current_dir()?.join(working_dir)
            } else {
                working_dir.clone()
            }
        } else {
            env::current_dir()?
        };

        while let Some(input_value) = input_values.next()? {
            let global_counter = if global_counter_used {
                global_counter_generator.next()
            } else {
                0
            };

            let local_counter = if local_counter_used {
                local_counter_generator.next(input_value)
            } else {
                0
            };

            let regex_captures = if regex_capture_used {
                regex_solver.eval(input_value)
            } else {
                None
            };

            let context = eval::Context {
                working_dir: &working_dir,
                global_counter,
                local_counter,
                regex_captures,
            };

            let output_value = match pattern.eval(input_value, &context) {
                Ok(value) => value,
                Err(error) => {
                    write_pattern_error(&mut io.stderr(), &error, raw_pattern)?;
                    if cli.fail_at_end {
                        exit_code = EXIT_CODE_EVAL_ERROR;
                        continue;
                    } else {
                        return Ok(EXIT_CODE_EVAL_ERROR);
                    }
                }
            };

            output_values.write(input_value, &output_value)?;
        }
    } else {
        while let Some(value) = input_values.next()? {
            output_values.write(value, value)?;
        }
    };

    io.stdout().flush()?; // output::Values may not do flush if there is no trailing delimiter.
    Ok(exit_code)
}
