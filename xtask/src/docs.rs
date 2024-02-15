use crate::command::Adapter;
use crate::command::BaseArg;
use crate::command::GlobalArgs;
use crate::command::OptionalArg;
use crate::command::PositionalArg;
use anyhow::anyhow;
use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub fn generate_summary(command: &Adapter<'_>, path: &Path) -> Result<()> {
    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    let start_marker = "<!--[GENERATED_CONTENT_START]-->";
    let end_marker = "<!--[GENERATED_CONTENT_END]-->";

    let start_pos = match content.find(start_marker) {
        Some(pos) => pos + start_marker.len(),
        None => {
            return Err(anyhow!(
                "'{}' did not contain '{}' marker",
                path.to_string_lossy(),
                start_marker
            ))
        }
    };

    let Some(end_pos) = content.find(end_marker) else {
        return Err(anyhow!(
            "'{}' did not contain '{}' marker",
            path.to_string_lossy(),
            end_marker
        ));
    };

    let mut file = File::create(path)?;
    file.write_all(&content.as_bytes()[..start_pos])?;

    writeln!(&mut file)?;
    writeln!(&mut file)?;
    write_summary(&mut file, command)?;
    writeln!(&mut file)?;

    file.write_all(&content.as_bytes()[end_pos..])?;

    Ok(())
}

fn write_summary(writer: &mut impl Write, command: &Adapter<'_>) -> Result<()> {
    writeln!(
        writer,
        "- [{}](./reference/{}.md)",
        command.full_name(),
        command.file_stem()
    )?;

    if let Some(subcommands) = command.subcommands() {
        for subcommand in &subcommands {
            write_summary(writer, subcommand)?;
        }
    }

    Ok(())
}

pub fn generate_reference(command: &Adapter<'_>, dir: &Path) -> Result<()> {
    let path = dir.join(command.file_stem()).with_extension("md");
    write_reference(&mut File::create(path)?, command)?;

    if let Some(subcommands) = command.subcommands() {
        for subcommand in subcommands {
            generate_reference(&subcommand, dir)?;
        }
    }

    Ok(())
}

fn write_reference(writer: &mut impl Write, command: &Adapter<'_>) -> Result<()> {
    write_heading(writer, command)?;
    write_usage(writer, command)?;

    if let Some(subcommands) = command.subcommands() {
        write_subcommands(writer, &subcommands)?;
    }

    if let Some(positionals) = command.pos_args() {
        write_pos_args(writer, &positionals)?;
    }

    if let Some(options) = command.opt_args() {
        write_opt_args(writer, &options)?;
    }

    if let Some(options) = command.global_opt_args() {
        write_global_opt_args(writer, &options)?;
    }

    Ok(())
}

fn write_heading(writer: &mut impl Write, command: &Adapter<'_>) -> Result<()> {
    writeln!(writer, "# {}", command.full_name())?;
    writeln!(writer)?;
    writeln!(writer, "{}", command.description()?)?;
    Ok(())
}

fn write_usage(writer: &mut impl Write, command: &Adapter<'_>) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "## Usage")?;
    writeln!(writer)?;
    writeln!(writer, "```")?;
    write!(writer, "{}", command.full_name())?;

    for arg in command.synopsis_args() {
        if arg.is_required() {
            write!(writer, " <{}>", arg.name())?;
        } else {
            write!(writer, " [{}]", arg.name())?;
        }
        if arg.is_many() {
            writeln!(writer, "...")?;
        }
    }

    writeln!(writer)?;
    writeln!(writer, "```")?;
    Ok(())
}

fn write_subcommands(writer: &mut impl Write, subcommands: &[Adapter<'_>]) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "## Commands")?;
    writeln!(writer)?;
    writeln!(writer, "<dl>")?;

    for subcommand in subcommands {
        write_subcommand(writer, subcommand)?;
    }

    writeln!(writer, "</dl>")?;
    Ok(())
}

fn write_subcommand(writer: &mut impl Write, subcommands: &Adapter<'_>) -> Result<()> {
    writeln!(
        writer,
        "<dt><a href=\"{}.html\"><code>{}</code></a></dt>",
        subcommands.file_stem(),
        subcommands.name()
    )?;
    writeln!(writer, "<dd>{}</dd>", subcommands.short_description()?)?;
    Ok(())
}

fn write_pos_args(writer: &mut impl Write, args: &[PositionalArg<'_>]) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "## Arguments")?;
    writeln!(writer)?;
    writeln!(writer, "<dl>")?;

    for arg in args {
        write_pos_arg(writer, arg)?;
    }

    writeln!(writer, "</dl>")?;
    Ok(())
}

fn write_pos_arg(writer: &mut impl Write, arg: &PositionalArg<'_>) -> Result<()> {
    write!(writer, "<dt><code>")?;

    if arg.is_required() {
        write!(writer, "<{}>", arg.name())?;
    } else {
        write!(writer, "[{}]", arg.name())?;
    }

    if arg.is_many() {
        write!(writer, "...")?;
    }

    writeln!(writer, "</code></dt>")?;
    write_arg(writer, arg.base())
}

fn write_opt_args(writer: &mut impl Write, args: &[OptionalArg<'_>]) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "## Options")?;
    writeln!(writer)?;
    writeln!(writer, "<dl>")?;

    for arg in args {
        write_opt_arg(writer, arg)?;
    }

    writeln!(writer, "</dl>")?;
    Ok(())
}

fn write_opt_arg(writer: &mut impl Write, arg: &OptionalArg<'_>) -> Result<()> {
    writeln!(writer)?;
    write!(writer, "<dt><code>{}", arg.names().join(", "))?;

    for value_name in arg.value_names() {
        write!(writer, " <{value_name}>")?;
    }

    writeln!(writer, "</code></dt>")?;
    write_arg(writer, arg.base())
}

fn write_global_opt_args(writer: &mut impl Write, args: &GlobalArgs<'_>) -> Result<()> {
    writeln!(writer)?;
    writeln!(writer, "## Global options")?;

    if let Some(command) = &args.inherited_from {
        writeln!(writer)?;
        writeln!(
            writer,
            "See [{} reference]({}.md#global-options) for list of additional global options.",
            command.full_name(),
            command.file_stem()
        )?;
    }

    if let Some(args) = &args.own {
        writeln!(writer)?;
        writeln!(writer, "<dl>")?;

        for arg in args {
            write_opt_arg(writer, arg)?;
        }

        writeln!(writer, "</dl>")?;
    }

    Ok(())
}

fn write_arg(writer: &mut impl Write, arg: &BaseArg<'_>) -> Result<()> {
    writeln!(writer, "<dd>")?;
    writeln!(writer, "{}", arg.description()?)?;

    if let Some(possible_values) = arg.possible_values() {
        writeln!(writer)?;
        writeln!(writer, "Possible values:")?;
        writeln!(writer)?;

        for value in possible_values {
            writeln!(writer, " - `{}` - {}", value.name(), value.description()?)?;
        }
    }

    if let Some(default_value) = arg.default_value() {
        writeln!(writer)?;
        writeln!(writer, "Default value: `{default_value}`")?;
    }

    if let Some(env_var) = arg.env_var() {
        writeln!(writer)?;
        writeln!(
            writer,
            "Can be also set using `{env_var}` environment variable."
        )?;
    }

    writeln!(writer, "</dd>")?;
    Ok(())
}
