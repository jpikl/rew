use crate::cli::Cli;
use crate::output::write_pattern_error;
use crate::pattern::{eval, Pattern};
use common::input::Delimiter as InputDelimiter;
use common::io::Io;
use common::output::write_error;
use common::run::{Result, Runner, EXIT_CODE_OK};
use std::env;

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
mod regex;
mod utils;

const EXIT_CODE_PARSE_ERROR: i32 = 3;
const EXIT_CODE_EVAL_ERROR: i32 = 4;

fn main() {
    Runner::new().exec(run);
}

fn run<'a, IO: Io<'a>>(cli: &'a Cli, io: &'a IO) -> Result {
    let pattern = match Pattern::parse(&cli.pattern, cli.escape) {
        Ok(pattern) => pattern,
        Err(error) => {
            write_pattern_error(&mut io.stderr(), &error, &cli.pattern)?;
            return Ok(EXIT_CODE_PARSE_ERROR);
        }
    };

    if cli.explain {
        pattern.explain(&mut io.stdout())?;
        return Ok(0);
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
        output::PathMode::InOutPretty
    } else {
        let output_delimiter = if cli.print_raw {
            None
        } else if cli.print_nul {
            Some('\0')
        } else {
            Some('\n')
        };
        if cli.batch {
            output::PathMode::InOut(output_delimiter)
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
                        exit_code = EXIT_CODE_EVAL_ERROR;
                        continue;
                    } else {
                        return Ok(EXIT_CODE_EVAL_ERROR);
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
                write_pattern_error(&mut io.stderr(), &error, &cli.pattern)?;
                if cli.fail_at_end {
                    exit_code = EXIT_CODE_EVAL_ERROR;
                    continue;
                } else {
                    return Ok(EXIT_CODE_EVAL_ERROR);
                }
            }
        };

        output_paths.write(path, &output_path)?;
    }

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::io::mem::OutputChunk;
    use common::run::TestRunner;
    use termcolor::Color;

    #[test]
    fn returns_parse_error_code() {
        let runner = TestRunner::new(&["{"], &[]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_PARSE_ERROR);
        assert_eq!(runner.stdout(), vec![]);
        assert_eq!(
            runner.stderr(),
            vec![
                OutputChunk::color(Color::Red, "error:"),
                OutputChunk::plain(" Invalid pattern: Expected variable after \'{\'\n\n{\n"),
                OutputChunk::bold_color(Color::Red, " ^"),
                OutputChunk::plain("\n"),
            ]
        );
    }

    #[test]
    fn prints_explanation() {
        let runner = TestRunner::new(&["--explain", "_"], &[]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(
            runner.stdout(),
            vec![
                OutputChunk::bold_color(Color::Green, "_"),
                OutputChunk::plain("\n"),
                OutputChunk::bold_color(Color::Green, "^"),
                OutputChunk::plain("\n\n"),
                OutputChunk::color(Color::Green, "Constant \'_\'"),
                OutputChunk::plain("\n")
            ]
        );
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn uses_paths_from_args_over_stdin() {
        let runner = TestRunner::new(&["_{p}_", "123", "456"], &b"abc\ndef"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(runner.stdout(), vec![OutputChunk::plain("_123_\n_456_\n")]);
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn reads_prints_lines() {
        let runner = TestRunner::new(&["_{p}_"], &b"abc\n\0def"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(
            runner.stdout(),
            vec![OutputChunk::plain("_abc_\n_\0def_\n")]
        );
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn reads_prints_nulls() {
        let runner = TestRunner::new(&["-z", "-Z", "_{p}_"], &b"abc\n\0def"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(
            runner.stdout(),
            vec![OutputChunk::plain("_abc\n_\0_def_\0")]
        );
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn reads_prints_raw() {
        let runner = TestRunner::new(&["-r", "-R", "_{p}_"], &b"abc\n\0def"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(runner.stdout(), vec![OutputChunk::plain("_abc\n\0def_")]);
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn reads_prints_batch() {
        let runner = TestRunner::new(&["-b", "_{p}_"], &b"abc"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(runner.stdout(), vec![OutputChunk::plain("<abc\n>_abc_\n")]);
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn reads_prints_pretty() {
        let runner = TestRunner::new(&["-p", "_{p}_"], &b"abc"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(
            runner.stdout(),
            vec![
                OutputChunk::color(Color::Blue, "abc"),
                OutputChunk::plain(" -> "),
                OutputChunk::color(Color::Green, "_abc_\n")
            ]
        );
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn uses_file_name_regex() {
        let runner = TestRunner::new(&["-e", "([0-9]+)", "{1}"], &b"dir01/file02"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(runner.stdout(), vec![OutputChunk::plain("02\n")]);
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn uses_path_regex() {
        let runner = TestRunner::new(&["-E", "([0-9]+)", "{1}"], &b"dir01/file02"[..]).unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(runner.stdout(), vec![OutputChunk::plain("01\n")]);
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn uses_local_counter() {
        let runner = TestRunner::new(
            &["--lc-init=2", "--lc-step=3", "{p}.{c}"],
            &b"a/a\na/b\nb/a\nb/b"[..],
        )
        .unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(
            runner.stdout(),
            vec![OutputChunk::plain("a/a.2\na/b.5\nb/a.2\nb/b.5\n")]
        );
        assert_eq!(runner.stderr(), vec![]);
    }

    #[test]
    fn uses_global_counter() {
        let runner = TestRunner::new(
            &["--gc-init=2", "--gc-step=3", "{p}.{C}"],
            &b"a/a\na/b\nb/a\nb/b"[..],
        )
        .unwrap();
        assert_eq!(runner.exec(run), EXIT_CODE_OK);
        assert_eq!(
            runner.stdout(),
            vec![OutputChunk::plain("a/a.2\na/b.5\nb/a.8\nb/b.11\n")]
        );
        assert_eq!(runner.stderr(), vec![]);
    }
}
