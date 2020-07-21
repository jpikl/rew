use crate::cli::Cli;
use crate::input::Input;
use crate::output::Output;
use crate::pattern::{eval, Pattern};
use std::io;
use std::process;
use structopt::StructOpt;
use termcolor::ColorChoice;

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
            output.write_pattern_error(raw_pattern, &error)?;
            process::exit(2);
        }
    };

    if cli.explain {
        return output.write_explanation(&pattern);
    }

    let global_counter_used = pattern.uses_global_counter();
    let local_counter_used = pattern.uses_local_counter();
    let regex_captures_used = pattern.uses_regex_captures();

    let mut global_counter =
        counter::Global::new(cli.gc_init.unwrap_or(1), cli.gc_step.unwrap_or(1));

    let mut local_counter = counter::Local::new(cli.lc_init.unwrap_or(1), cli.lc_step.unwrap_or(1));

    let regex_capture = if let Some(regex) = &cli.regex {
        regex::Capture::of_file_name(regex)
    } else if let Some(regex) = &cli.regex_full {
        regex::Capture::of_full_path(regex)
    } else {
        regex::Capture::of_none()
    };

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

    // TODO nicer error message for utf error
    while let Some(path) = input.next()? {
        let global_counter = if global_counter_used {
            global_counter.next()
        } else {
            0
        };

        let local_counter = if local_counter_used {
            local_counter.next(path)
        } else {
            0
        };

        let regex_captures = if regex_captures_used {
            regex_capture.get(path)
        } else {
            None
        };

        let context = eval::Context {
            path,
            global_counter,
            local_counter,
            regex_captures,
        };

        let out_path = match pattern.eval(&context) {
            Ok(path) => path,
            Err(error) => {
                output.write_pattern_error(raw_pattern, &error)?;
                process::exit(3);
            }
        };

        output.write_path(&out_path)?;
    }

    Ok(())
}
