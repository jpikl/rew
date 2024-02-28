use crate::colors::Colorizer;
use crate::colors::GREEN;
use crate::colors::RED;
use crate::colors::RESET;
use crate::colors::YELLOW;
use crate::pager;
use anstream::stdout;
use anyhow::format_err;
use anyhow::Context;
use anyhow::Result;
use clap::crate_name;
use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;
use std::io;
use std::io::Write;
use std::panic::resume_unwind;
use std::thread;
use unicode_width::UnicodeWidthStr;

pub struct Example {
    pub text: &'static str,
    pub args: &'static [&'static str],
    pub input: &'static [&'static str],
    pub output: &'static [&'static str],
}

#[macro_export]
macro_rules! examples {
    ($($text:literal: { args: $args:expr, input: $input:expr, output: $output:expr, }),*,) => {
        &[$($crate::examples::Example { text: $text, args: $args, input: $input, output: $output }),*]
    };
    () => {
        &[]
    };
}

const ARG: &str = "examples";

pub fn augment_args(mut command: Command) -> Command {
    let required_arg_ids = command
        .get_arguments()
        .filter(|arg| arg.is_required_set())
        .map(Arg::get_id)
        .cloned()
        .collect::<Vec<_>>();

    command = command.arg(
        Arg::new(ARG)
            .long("examples")
            .action(ArgAction::SetTrue)
            .help("Print examples of the command usage")
            .display_order(1000),
    );

    // Do not require other arguments when `--examples` is present
    for arg_id in required_arg_ids {
        command = command.mut_arg(arg_id, |arg| {
            arg.required(false).required_unless_present(ARG)
        });
    }

    command
}

pub fn is_arg_set(matches: &ArgMatches) -> bool {
    matches.get_flag(ARG)
}

pub fn print(command: &'static str, examples: &'static [Example]) -> Result<()> {
    if let Some(mut pager) = pager::open().context("could not start pager process")? {
        let mut stdin = pager.stdin.take().expect("could not get pager stdin");

        let thread = thread::spawn(move || {
            write(&mut stdin, command, examples).context("could not write to pager stdin")
        });

        let status = pager.wait().context("could not wait for pager to finish")?;
        if !status.success() {
            let code = status.code().unwrap_or_default();
            let err = format_err!("pager exited with code {RED}{code}{RESET}");
            return Err(err);
        }

        thread.join().map_err(resume_unwind)?.map(Into::into)
    } else {
        write(&mut stdout().lock(), command, examples)
    }
}

fn write(writer: &mut impl Write, subcmd: &str, examples: &[Example]) -> Result<()> {
    for example in examples {
        write_example(writer, subcmd, example)?;
    }
    Ok(())
}

fn write_example(writer: &mut impl Write, subcmd: &str, example: &Example) -> io::Result<()> {
    writeln!(writer)?;

    let colorizer = Colorizer {
        quote_char: '`',
        quote_color: YELLOW,
    };

    colorizer.write(writer, example.text)?;

    writeln!(writer)?;
    writeln!(writer)?;

    write_command(writer, subcmd, example.args)?;
    write_io(writer, example.input, example.output)
}

fn write_command(writer: &mut impl Write, subcmd: &str, args: &[&str]) -> io::Result<()> {
    let prefix = "$ ";
    let cmd = crate_name!();
    let args = normalize_args(args);

    let args_width = args.iter().fold(0, |sum, arg| sum + arg.width());
    let code_width = cmd.width() + subcmd.width() + args_width + args.len() + 1;
    let padding_width = 40usize.saturating_sub(prefix.width() + code_width);
    let width = prefix.width() + code_width + padding_width;

    // https://en.wikipedia.org/wiki/Box-drawing_character

    writeln!(writer, " {GREEN}╭{}╮{RESET}", "─".repeat(width))?;
    write!(writer, " {GREEN}│{prefix}{RESET}{cmd} {subcmd}")?;

    for arg in &args {
        if is_quoted(arg) {
            write!(writer, " {YELLOW}{arg}{RESET}")?;
        } else {
            write!(writer, " {arg}")?;
        }
    }

    writeln!(writer, "{}{GREEN}│{RESET}", " ".repeat(padding_width))?;
    writeln!(writer, " {GREEN}╰{}╯{RESET}", "─".repeat(width))
}

fn write_io(writer: &mut impl Write, input: &[&str], output: &[&str]) -> io::Result<()> {
    if input.is_empty() && output.is_empty() {
        return Ok(());
    }

    let input = normalize_lines(input);
    let output = normalize_lines(output);

    let input_label = "stdin:";
    let output_label = "stdout:";

    let max_lines = input.len().max(output.len());

    for i in 0..max_lines {
        if !input.is_empty() {
            write_io_label(writer, input_label, i)?;
            write_io_line(writer, &input, i)?;
        }

        if !input.is_empty() && !output.is_empty() {
            write!(writer, "  ")?;
        }

        if !output.is_empty() {
            write_io_label(writer, output_label, i)?;
            write_io_line(writer, &output, i)?;
        }

        writeln!(writer)?;
    }

    Ok(())
}

fn write_io_label(writer: &mut impl Write, label: &str, index: usize) -> io::Result<()> {
    write!(writer, "  ")?;

    if index == 0 {
        write!(writer, "{GREEN}{label}{RESET}")
    } else {
        write!(writer, "{}", " ".repeat(label.width()))
    }
}

fn write_io_line(writer: &mut impl Write, lines: &[String], index: usize) -> io::Result<()> {
    write!(writer, " ")?;

    if let Some(line) = lines.get(index) {
        write!(writer, "{GREEN}\"{RESET}{line}{GREEN}\"{RESET}{RESET}",)?;
        writer.write_all(" ".repeat(max_line_width(lines) - line.width()).as_bytes())
    } else {
        write!(writer, "{}", " ".repeat(max_line_width(lines) + 2))
    }
}

fn normalize_args(args: &[&str]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            let arg = arg.replace('\'', "\"");

            if arg.contains(' ') || arg.contains('|') || arg.contains('\\') {
                format!("'{arg}'")
            } else {
                arg
            }
        })
        .collect::<Vec<_>>()
}

fn normalize_lines(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.replace('\t', "    "))
        .collect::<Vec<_>>()
}

fn max_line_width(lines: &[String]) -> usize {
    lines.iter().fold(0, |max, line| max.max(line.width()))
}

fn is_quoted(value: &str) -> bool {
    value.starts_with('\'')
        || value.starts_with('"')
        || value.ends_with('\'')
        || value.ends_with('"')
}
