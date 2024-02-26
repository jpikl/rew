use crate::colors::Colorizer;
use crate::colors::GREEN;
use crate::colors::RESET;
use crate::colors::YELLOW;
use anstream::stdout;
use anyhow::Result;
use clap::crate_name;
use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

pub struct Example {
    pub name: &'static str,
    pub args: &'static [&'static str],
    pub input: &'static [&'static str],
    pub output: &'static [&'static str],
}

#[macro_export]
macro_rules! examples {
    ($($name:literal: { args: $args:expr, input: $input:expr, output: $output:expr, }),*,) => {
        || vec![$($crate::examples::Example { name: $name, args: $args, input: $input, output: $output }),*]
    };
    () => {
        Vec::new
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

pub fn is_set(matches: &ArgMatches) -> bool {
    matches.get_flag(ARG)
}

pub fn print(command: &str, examples: &[Example]) -> Result<()> {
    let mut stdout = stdout().lock();

    for example in examples {
        write_example(&mut stdout, command, example)?;
    }

    Ok(())
}

fn write_example(writer: &mut impl Write, subcmd: &str, example: &Example) -> Result<()> {
    writeln!(writer)?;

    let name_colorizer = Colorizer {
        quote_char: '`',
        quote_color: YELLOW,
    };

    name_colorizer.write(writer, example.name)?;

    writeln!(writer)?;
    writeln!(writer)?;

    let input = normalize_lines(example.input);
    let output = normalize_lines(example.output);

    let cmd_name = crate_name!();
    let cmd_args = normalize_command_args(subcmd, example.args);

    let mut width = 40;
    width = width.max(lines_width(&input));
    width = width.max(lines_width(&output));
    width = width.max(command_width(cmd_name, &cmd_args));

    // https://en.wikipedia.org/wiki/Box-drawing_character

    if input.is_empty() {
        write_horizontal_line(writer, '╭', '╮', width, Some("command"))?;
    } else {
        write_horizontal_line(writer, '╭', '╮', width, Some("stdin"))?;

        for line in &input {
            write_text_line(writer, width, line)?;
        }

        write_horizontal_line(writer, '├', '┤', width, Some("command"))?;
    }

    write_command_line(writer, width, cmd_name, &cmd_args)?;

    if !output.is_empty() {
        write_horizontal_line(writer, '├', '┤', width, Some("stdout"))?;

        for line in &output {
            write_text_line(writer, width, line)?;
        }
    }

    write_horizontal_line(writer, '╰', '╯', width, None)
}

fn write_horizontal_line(
    writer: &mut impl Write,
    start: char,
    end: char,
    width: usize,
    title: Option<&str>,
) -> Result<()> {
    write!(writer, " {GREEN}{start}")?;

    let remainder = if let Some(title) = title {
        write!(writer, "─[{title}]")?;
        width - title.width() - 3
    } else {
        width
    };

    writer.write_all("─".repeat(remainder).as_bytes())?;
    writeln!(writer, "{end}{RESET}")?;
    Ok(())
}

fn write_text_line(writer: &mut impl Write, width: usize, text: &str) -> Result<()> {
    let padding = " ".repeat(width - text.width());
    writeln!(writer, " {GREEN}│{RESET}{text}{padding}{GREEN}│{RESET}")?;
    Ok(())
}

fn write_command_line(
    writer: &mut impl Write,
    width: usize,
    name: &str,
    args: &[String],
) -> Result<()> {
    write!(writer, " {GREEN}│{RESET}{name}")?;

    for arg in args {
        if is_quoted(arg) {
            write!(writer, " {YELLOW}{arg}{RESET}")?;
        } else {
            write!(writer, " {arg}")?;
        }
    }

    let padding = " ".repeat(width - command_width(name, args));
    writeln!(writer, "{padding}{GREEN}│{RESET}")?;
    Ok(())
}

fn normalize_command_args(subcmd: &str, subcmd_args: &[&str]) -> Vec<String> {
    let mut args = Vec::new();
    args.push(subcmd.to_owned());

    for arg in subcmd_args {
        let arg = arg.replace('\'', "\"");

        if arg.contains(' ') || arg.contains('|') || arg.contains('\\') {
            args.push(format!("'{arg}'"));
        } else {
            args.push(arg);
        }
    }

    args
}

fn normalize_lines(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .map(|line| line.replace('\t', "   "))
        .collect::<Vec<_>>()
}

fn command_width(name: &str, args: &[String]) -> usize {
    let mut width = name.width();

    for arg in args {
        width += arg.width() + 1;
    }

    width
}

fn lines_width(lines: &[String]) -> usize {
    let mut width = 0;

    for line in lines {
        width = width.max(line.width());
    }

    width
}

fn is_quoted(value: &str) -> bool {
    value.starts_with('\'')
        || value.starts_with('"')
        || value.ends_with('\'')
        || value.ends_with('"')
}
