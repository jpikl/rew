use crate::cli::Cli;
use crate::output::write_pattern_error;
use crate::pattern::{eval, help, Pattern};
use common::input::Delimiter as InputDelimiter;
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};
use std::env;

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
#[cfg(test)]
mod testing;
mod utils;

const EXIT_CODE_PATTERN_PARSE_ERROR: i32 = 3;
const EXIT_CODE_PATTERN_EVAL_ERROR: i32 = 4;

fn main() {
    exec_run(run);
}

fn run(cli: &Cli, io: &Io) -> Result {
    if cli.help_pattern {
        help::write_pattern_help(&mut io.stdout())?;
        return Ok(EXIT_CODE_OK);
    }

    if cli.help_filters {
        help::write_filters_help(&mut io.stdout())?;
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
    } else {
        let output_delimiter = if cli.print_raw {
            None
        } else if cli.print_nul {
            Some('\0')
        } else {
            Some('\n')
        };
        if cli.bulk {
            output::Mode::Diff(output_delimiter)
        } else {
            output::Mode::Out(output_delimiter)
        }
    };

    let mut output_values = output::Values::new(io.stdout(), output_mode);
    let mut exit_code = EXIT_CODE_OK;

    if let Some(raw_pattern) = cli.pattern.as_ref() {
        let pattern = match Pattern::parse(raw_pattern, cli.escape) {
            Ok(pattern) => pattern,
            Err(error) => {
                write_pattern_error(&mut io.stderr(), &error, raw_pattern)?;
                return Ok(EXIT_CODE_PATTERN_PARSE_ERROR);
            }
        };

        if cli.explain {
            pattern.explain(&mut io.stdout())?;
            return Ok(EXIT_CODE_OK);
        }

        let global_counter_used = pattern.uses_global_counter();
        let local_counter_used = pattern.uses_local_counter();

        let mut global_counter_generator =
            counter::GlobalGenerator::new(cli.gc_init.unwrap_or(1), cli.gc_step.unwrap_or(1));

        let mut local_counter_generator =
            counter::LocalGenerator::new(cli.lc_init.unwrap_or(1), cli.lc_step.unwrap_or(1));

        let current_dir_buf = env::current_dir()?;
        let current_dir = current_dir_buf.as_path();

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

            let context = eval::Context {
                current_dir,
                global_counter,
                local_counter,
            };

            let output_value = match pattern.eval(input_value, &context) {
                Ok(value) => value,
                Err(error) => {
                    write_pattern_error(&mut io.stderr(), &error, raw_pattern)?;
                    if cli.fail_at_end {
                        exit_code = EXIT_CODE_PATTERN_EVAL_ERROR;
                        continue;
                    } else {
                        return Ok(EXIT_CODE_PATTERN_EVAL_ERROR);
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

    Ok(exit_code)
}
