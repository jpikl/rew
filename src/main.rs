use crate::cli::Cli;
use crate::input::Input;
use crate::output::Output;
use crate::pattern::{eval, Pattern};
use std::collections::HashMap;
use std::io;
use std::process;
use structopt::StructOpt;
use termcolor::ColorChoice;

mod cli;
mod input;
mod output;
mod pattern;
mod utils;

const EXIT_PARSE_ERROR: i32 = 2;
const EXIT_EVAL_ERROR: i32 = 3;

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

    let output_delimiter = if cli.print_raw {
        None
    } else if cli.print_nul {
        Some('\0')
    } else {
        Some('\n')
    };

    let raw_pattern = &cli.pattern;
    let mut output = Output::new(output_colors, output_delimiter);

    let pattern = match Pattern::parse(raw_pattern, cli.escape) {
        Ok(pattern) => pattern,
        Err(error) => {
            output.write_parse_error(raw_pattern, &error)?;
            process::exit(EXIT_PARSE_ERROR);
        }
    };

    if cli.explain {
        return output.write_explanation(&pattern);
    }

    let mut input = if cli.paths.is_empty() {
        let input_delimiter = if cli.read_raw {
            None
        } else if cli.read_nul {
            Some(0)
        } else {
            Some(b'\n')
        };
        Input::from_stdin(input_delimiter)
    } else {
        Input::from_args(cli.paths.as_slice())
    };

    let global_counter_used = pattern.uses_global_counter();
    let local_counter_used = pattern.uses_local_counter();
    let regex_captures_used = pattern.uses_regex_captures();

    let mut global_counter = cli.gc_init.unwrap_or(1);
    let global_counter_step = cli.gc_step.unwrap_or(1);

    let mut local_counters = HashMap::new();
    let local_counter_start = cli.lc_init.unwrap_or(1);
    let local_counter_step = cli.lc_step.unwrap_or(1);

    while let Some(src_path) = input.next()? {
        // TODO nicer error message for utf error
        let local_counter = if local_counter_used {
            if let Some(directory) = src_path.parent() {
                let directory_buf = directory.to_path_buf();
                if let Some(local_counter) = local_counters.get_mut(&directory_buf) {
                    *local_counter += local_counter_step;
                    *local_counter
                } else {
                    local_counters.insert(directory_buf, local_counter_start);
                    local_counter_start
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

        let context = eval::Context {
            path: src_path,
            global_counter,
            local_counter,
            regex_captures,
        };

        let dst_path = match pattern.eval(&context) {
            Ok(path) => path,
            Err(error) => {
                output.write_eval_error(raw_pattern, &error)?;
                process::exit(EXIT_EVAL_ERROR);
            }
        };

        output.write_path(&dst_path)?;

        if global_counter_used {
            global_counter += global_counter_step;
        }
    }

    Ok(())
}
