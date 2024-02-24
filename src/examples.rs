use anyhow::Result;
use bstr::ByteSlice;
use clap::crate_name;
use clap::Arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;
use std::io::Write;

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
    let mut stdout = anstream::stdout().lock();

    for example in examples {
        print_example(&mut stdout, command, example)?;
    }

    Ok(())
}

fn print_example(writer: &mut impl Write, command: &str, example: &Example) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "{}", example.name)?;
    writeln!(writer)?;

    let mut buffer = Vec::new();
    print_code(&mut buffer, command, example)?;
    buffer = buffer.replace("\t", "    ");

    let code = String::from_utf8_lossy(&buffer);
    let width = code
        .lines()
        .fold(0, |max_len, line| max_len.max(line.chars().count()));

    writeln!(writer, " ╭{}╮", "─".repeat(width))?;

    for line in code.lines() {
        let padding = " ".repeat(width - line.chars().count());
        writeln!(writer, " │{line}{padding}│")?;
    }

    writeln!(writer, " ╰{}╯", "─".repeat(width))?;

    Ok(())
}

fn print_code(writer: &mut impl Write, command: &str, example: &Example) -> Result<()> {
    if !example.input.is_empty() {
        let mut input = example.input.iter();

        if let Some(line) = input.next() {
            writeln!(writer, "$ echo '{line}' > input")?;
        }

        for line in input {
            writeln!(writer, "$ echo '{line}' >> input")?;
        }

        writeln!(writer)?;
    }

    write!(writer, "$ {} {command}", crate_name!())?;

    for arg in example.args {
        if arg.contains(' ') || arg.contains('|') || arg.contains('\\') {
            write!(writer, " '{arg}'")?;
        } else {
            write!(writer, " {arg}")?;
        }
    }

    if !example.input.is_empty() {
        write!(writer, " < input")?;
    }

    writeln!(writer)?;
    writeln!(writer)?;
    writeln!(writer, "{}", example.output.join("\n"))?;
    Ok(())
}
