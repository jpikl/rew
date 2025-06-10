use super::ArgUsage;
use super::CallChain;
use super::Command;
use super::CommandBuilder;
use super::CommandItem;
use super::Context;
use super::ErrorKind;
use super::Flag;
use super::FlagBuilder;
use super::Group;
use super::Pos;
use super::PosBuilder;
use super::QUOTE_END;
use super::QUOTE_START;
use super::ValueSource;
use crate::cli::HIGHLIGHT_END;
use crate::cli::HIGHLIGHT_START;
use crate::cli::Highlight;
use crate::cli::SECTION_END;
use crate::cli::SECTION_START;
use std::fmt::Display;
use std::io::Write;

pub const HELP: Flag = FlagBuilder::new()
    .short('h')
    .long("help")
    .description("Print short help `-h` or detailed help `--help`.")
    .run(print_help)
    .err_hint(print_err_hint)
    .done();

const HELP_ARG: Pos<String> = PosBuilder::new()
    .name("COMMAND")
    .description("Subcommand chain")
    .description_ex(&["If none is provided, print help for the main command."])
    .multiple()
    .done();

pub const HELP_CMD: Command = CommandBuilder::new()
    .name("help")
    .description("Print help for a subcommand.")
    .description_ex(&["Provides the same output as `--help` flag."])
    .positionals(&[&HELP_ARG.arg])
    .run(print_subcommand_help)
    .done();

fn print_subcommand_help(ctx: &Context) -> Result<(), ErrorKind<'static>> {
    let Some(mut command) = ctx.commands.parent() else {
        return Ok(());
    };
    for name in ctx.args.iter_ref(&HELP_ARG) {
        if let Some(child) = command.subcommand(&name) {
            command = child;
        } else {
            return Err(ErrorKind::UnknownSubcommand(name.to_string().into()));
        }
    }
    command.print_help(&mut anstream::stdout().lock(), &ctx.calls, true)?;
    Ok(())
}

fn print_help<'a>(ctx: &Context<'a>, usage: &ArgUsage) -> Result<(), ErrorKind<'a>> {
    let long = matches!(usage.source, ValueSource::LongOption);
    let command = ctx.commands.current();
    command.print_help(&mut anstream::stdout().lock(), &ctx.calls, long)?;
    Ok(())
}

fn print_err_hint(ctx: &Context, err: &ErrorKind, out: &mut dyn Write) -> std::io::Result<()> {
    if err.is_invalid_usage() {
        writeln!(out, "Try {QUOTE_START}{} -h{QUOTE_END} for program usage.", ctx.calls)?;
        writeln!(
            out,
            "You can get more detailed usage with {QUOTE_START}{} --help{QUOTE_END}.",
            ctx.calls
        )?;
    }
    Ok(())
}

impl Command<'_> {
    pub fn print_help(&self, writer: &mut impl Write, calls: &CallChain, long: bool) -> std::io::Result<()> {
        writeln!(writer, "{}", Highlight(self.description))?;

        if long {
            for description in self.description_ex {
                writeln!(writer, "{}", Highlight(description))?;
            }
        }

        writeln!(writer)?;
        write!(
            writer,
            "{SECTION_START}Usage:{SECTION_END} {HIGHLIGHT_START}{}{HIGHLIGHT_END}",
            calls
        )?;

        for param in self.params() {
            write!(writer, " {param}")?;
        }

        writeln!(writer)?;

        for (group, commands) in self.grouped_subcommands() {
            print_item_group(writer, group, &commands, long, false)?;
        }

        for (group, positionals) in self.grouped_positionals() {
            print_item_group(writer, group, &positionals, long, true)?;
        }

        for (group, options) in self.grouped_options() {
            print_item_group(writer, group, &options, long, true)?;
        }

        Ok(())
    }
}

fn print_item_group<T: CommandItem + Display>(
    writer: &mut impl Write,
    group: &Group,
    items: &[&T],
    long: bool,
    with_params: bool,
) -> std::io::Result<()> {
    writeln!(writer)?;
    writeln!(writer, "{SECTION_START}{}:{SECTION_END}", group.name)?;

    if long {
        let mut add_newline = false;

        for item in items {
            if add_newline {
                writeln!(writer)?;
            } else {
                add_newline = true;
            }

            write!(writer, "  {HIGHLIGHT_START}{item}{HIGHLIGHT_END}")?;

            if with_params {
                for param in item.params() {
                    write!(writer, " {param}")?;
                }
            }

            writeln!(writer)?;
            writeln!(writer, "          {}", Highlight(item.description()))?;

            for description in item.description_ex() {
                writeln!(writer, "          {}", Highlight(description))?;
            }

            if !item.enum_items().is_empty() {
                writeln!(writer)?;
                writeln!(writer, "          Values:")?;

                for enum_item in item.enum_items() {
                    if let Some((main_desc, descriptions)) = enum_item.description.split_first() {
                        writeln!(
                            writer,
                            "          - {HIGHLIGHT_START}{}{HIGHLIGHT_END}: {main_desc}",
                            enum_item.name
                        )?;

                        let padding = " ".repeat(enum_item.name.chars().count());

                        for description in descriptions {
                            writeln!(writer, "              {padding}{}", Highlight(description))?;
                        }
                    } else {
                        writeln!(writer, "          - {HIGHLIGHT_START}{}{HIGHLIGHT_END}", enum_item.name)?;
                    }
                }
            }

            if item.default().is_some() || item.environment().is_some() {
                writeln!(writer)?;
            }

            if let Some(default) = item.default() {
                writeln!(writer, "          Default value: {default}")?;
            }

            if let Some(environment) = item.environment() {
                writeln!(writer, "          Environment: {environment}")?;
            }
        }
    } else {
        let width = items
            .iter()
            .map(|item| get_usage_len(*item, with_params))
            .max()
            .unwrap_or_default();

        for item in items {
            write!(writer, "  {HIGHLIGHT_START}{item}{HIGHLIGHT_END}")?;

            if with_params {
                for param in item.params() {
                    write!(writer, " {param}")?;
                }
            }

            write!(writer, "{}  ", " ".repeat(width - get_usage_len(*item, with_params)))?;

            if let Some(default) = item.default() {
                if let Some(description) = item.description().strip_suffix('.') {
                    writeln!(writer, "{} (default: {}).", Highlight(description), default)?;
                } else {
                    writeln!(writer, "{} (default: {})", Highlight(item.description()), default)?;
                }
            } else {
                writeln!(writer, "{}", Highlight(item.description()))?;
            }
        }
    }

    Ok(())
}

fn get_usage_len<T: CommandItem + Display>(item: &T, with_params: bool) -> usize {
    let mut len = item.to_string().chars().count();
    if with_params {
        for param in item.params() {
            len += param.chars().count() + 1;
        }
    }
    len
}
