use super::CommandItem;
use super::Group;
use super::OptArg;
use super::PosArg;
use super::args::Args;
use super::builders::FlagBuilder;
use super::types::Command;
use super::types::Flag;
use anyhow::bail;
use std::io::Write;

pub const HELP: Flag = FlagBuilder::new("help")
    .short('h')
    .long("help")
    .description("Print short help (-h) or detailed help (--help)")
    .done();

pub const VERSION: Flag = FlagBuilder::new("version")
    .short('V')
    .long("version")
    .description("Print version")
    .done();

#[derive(Debug)]
pub struct Runner<'a> {
    pub binary: String,
    pub command: &'a Command,
    pub parents: Vec<&'a Command>,
    pub args: Args,
}

impl Runner<'_> {
    pub fn run(self) -> anyhow::Result<()> {
        if self.args.get(&HELP) {
            self.print_help(std::io::stdout().lock(), self.args.is_long(&HELP))?;
            return Ok(());
        }

        if self.args.get(&VERSION) {
            self.print_version(std::io::stdout().lock())?;
            return Ok(());
        }

        if !self.command.commands.is_empty() {
            bail!("Missing command");
        }

        for pos in self.command.positionals {
            if pos.required && !self.args.has(pos) {
                bail!("Missing required argument {pos}");
            }
        }

        (self.command.run)(self.args)
    }

    fn print_help(&self, mut writer: impl Write, long: bool) -> std::io::Result<()> {
        writeln!(writer, "{}", self.command.description)?;

        if long {
            for description in self.command.description_ex {
                writeln!(writer, "{description}")?;
            }
        }

        writeln!(writer)?;
        writeln!(
            writer,
            "Usage: {}{}",
            self.binary,
            self.command.usage_params()
        )?;

        for (group, commands) in self.command.grouped_commands() {
            print_item_group(&mut writer, group, &commands, long)?;
        }

        for (group, positionals) in self.command.grouped_positionals() {
            print_item_group(&mut writer, group, &positionals, long)?;
        }

        for (group, options) in self.command.grouped_options() {
            print_item_group(&mut writer, group, &options, long)?;
        }

        Ok(())
    }

    fn print_version(&self, mut writer: impl Write) -> std::io::Result<()> {
        for command in &self.parents {
            write!(writer, "{} ", command.name)?;
        }

        write!(writer, "{}", self.command.name)?;

        if let Some(version) = self.command.version {
            write!(writer, " {version}")?;
        }

        writeln!(writer)
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
        group_items(self.commands)
    }
}

fn print_item_group<T: CommandItem>(
    writer: &mut impl Write,
    group: &Group,
    items: &[&T],
    long: bool,
) -> std::io::Result<()> {
    writeln!(writer)?;
    writeln!(writer, "{}:", group.name)?;

    if long {
        let mut add_newline = false;

        for item in items {
            if add_newline {
                writeln!(writer)?;
            } else {
                add_newline = true;
            }

            writeln!(writer, "  {}", item.usage())?;
            writeln!(writer, "          {}", item.description())?;

            for description in item.description_ex() {
                writeln!(writer, "          {}", description)?;
            }

            if !item.enum_items().is_empty() {
                writeln!(writer)?;
                writeln!(writer, "          Values:")?;

                for enum_item in item.enum_items() {
                    if let Some((main_desc, descriptions)) = enum_item.description.split_first() {
                        writeln!(writer, "          - {}: {main_desc}", enum_item.name)?;
                        let padding = " ".repeat(enum_item.name.chars().count());

                        for description in descriptions {
                            writeln!(writer, "              {padding}{description}",)?;
                        }
                    } else {
                        writeln!(writer, "          - {}", enum_item.name)?;
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
            .map(|item| item.usage().chars().count())
            .max()
            .unwrap_or_default();

        for item in items {
            let usage = item.usage();
            let padding = " ".repeat(width - usage.len());
            writeln!(writer, "  {usage}{padding}  {}", item.description())?;
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
