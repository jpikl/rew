use super::CallChain;
use super::CommandItem;
use super::Error;
use super::ErrorKind;
use super::Group;
use super::OptArg;
use super::PosArg;
use super::USAGE_OPTIONS;
use super::args::Args;
use super::builders::FlagBuilder;
use super::types::Command;
use super::types::Flag;
use crate::colors::BOLD;
use crate::colors::RESET;
use anstream::stdout;
use std::fmt::Display;
use std::io::Write;

pub const HELP: Flag = FlagBuilder::new("help")
    .short('h')
    .long("help")
    .description("Print short help (-h) or detailed help (--help)")
    .group(&USAGE_OPTIONS)
    .done();

pub const VERSION: Flag = FlagBuilder::new("version")
    .short('V')
    .long("version")
    .description("Print version")
    .group(&USAGE_OPTIONS)
    .done();

#[derive(Debug)]
pub struct Runner<'a> {
    pub command: &'a Command,
    pub call_chain: CallChain,
    pub args: Args,
}

impl<'a> Runner<'a> {
    pub fn run(self) {
        if let Err(err) = self.try_run() {
            err.exit();
        }
    }

    pub fn try_run(self) -> Result<(), Error<'a>> {
        if self.args.get(&HELP) {
            return self
                .print_help(stdout().lock(), self.args.is_long(&HELP))
                .map_err(|err| self.err(ErrorKind::RunError(err.into())));
        }

        if self.args.get(&VERSION) {
            return self
                .print_version(stdout().lock())
                .map_err(|err| self.err(ErrorKind::RunError(err.into())));
        }

        if !self.command.subcommands.is_empty() {
            return Err(self.err(ErrorKind::MissingSubcommand));
        }

        for pos in self.command.positionals {
            if pos.required && !self.args.has(pos) {
                return Err(self.err(ErrorKind::MissingArgument(pos)));
            }
        }

        (self.command.run)(self.args).map_err(|err| Error {
            command: self.command,
            call_chain: self.call_chain,
            kind: ErrorKind::RunError(err),
        })
    }

    fn print_help(&self, mut writer: impl Write, long: bool) -> std::io::Result<()> {
        writeln!(writer, "{}", self.command.description)?;

        if long {
            for description in self.command.description_ex {
                writeln!(writer, "{description}")?;
            }
        }

        writeln!(writer)?;
        write!(writer, "{BOLD}Usage: {}{RESET}", self.call_chain)?;

        for param in self.command.params() {
            write!(writer, " {param}")?;
        }

        writeln!(writer)?;

        for (group, commands) in self.command.grouped_commands() {
            print_item_group(&mut writer, group, &commands, long, false)?;
        }

        for (group, positionals) in self.command.grouped_positionals() {
            print_item_group(&mut writer, group, &positionals, long, true)?;
        }

        for (group, options) in self.command.grouped_options() {
            print_item_group(&mut writer, group, &options, long, true)?;
        }

        Ok(())
    }

    fn print_version(&self, mut writer: impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self.command.name)?;

        if let Some(version) = self.command.version {
            write!(writer, " {version}")?;
        }

        writeln!(writer)
    }

    fn err(self, kind: ErrorKind<'a>) -> Error<'a> {
        Error {
            command: self.command,
            call_chain: self.call_chain,
            kind,
        }
    }
}

impl Command {
    fn grouped_options(&self) -> Vec<(&Group, Vec<&OptArg>)> {
        group_items(self.options)
    }

    fn grouped_positionals(&self) -> Vec<(&Group, Vec<&PosArg>)> {
        group_items(self.positionals)
    }

    fn grouped_commands(&self) -> Vec<(&Group, Vec<&Command>)> {
        group_items(self.subcommands)
    }

    pub fn help_option(&self) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.id == HELP.arg.id)
    }
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

fn print_item_group<T: CommandItem + Display>(
    writer: &mut impl Write,
    group: &Group,
    items: &[&T],
    long: bool,
    with_params: bool,
) -> std::io::Result<()> {
    writeln!(writer)?;
    writeln!(writer, "{BOLD}{}:{RESET}", group.name)?;

    if long {
        let mut add_newline = false;

        for item in items {
            if add_newline {
                writeln!(writer)?;
            } else {
                add_newline = true;
            }

            write!(writer, "  {BOLD}{item}{RESET}")?;

            if with_params {
                for param in item.params() {
                    write!(writer, " {param}")?;
                }
            }

            writeln!(writer)?;
            writeln!(writer, "          {}", item.description())?;

            for description in item.description_ex() {
                writeln!(writer, "          {}", description)?;
            }

            if !item.enum_items().is_empty() {
                writeln!(writer)?;
                writeln!(writer, "          Values:")?;

                for enum_item in item.enum_items() {
                    if let Some((main_desc, descriptions)) = enum_item.description.split_first() {
                        writeln!(writer, "          - {BOLD}{}{RESET}: {main_desc}", enum_item.name)?;

                        let padding = " ".repeat(enum_item.name.chars().count());

                        for description in descriptions {
                            writeln!(writer, "              {padding}{description}",)?;
                        }
                    } else {
                        writeln!(writer, "          - {BOLD}{}{RESET}", enum_item.name)?;
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
            write!(writer, "  {BOLD}{item}{RESET}")?;

            if with_params {
                for param in item.params() {
                    write!(writer, " {param}")?;
                }
            }

            writeln!(
                writer,
                "{}  {}",
                " ".repeat(width - get_usage_len(*item, with_params)),
                item.description()
            )?;
        }
    }

    Ok(())
}

fn group_items<T: CommandItem>(items: &[T]) -> Vec<(&Group, Vec<&T>)> {
    let mut grouped: Vec<(&Group, Vec<&T>)> = Vec::new();

    for item in items.iter() {
        let group = grouped.iter_mut().find(|(g, _)| *g == item.group());

        match group {
            Some((_, items)) => items.push(item),
            None => grouped.push((item.group(), vec![item])),
        }
    }

    grouped
}
