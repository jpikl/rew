use super::Arg;
use super::ArgId;
use super::CommandAlias;
use super::Context;
use super::EnumItem;
use super::ErrorKind;
use super::Group;
use super::OptArg;
use super::PosArg;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub description_ex: &'a [&'a str],
    pub version: Option<&'a str>,
    pub group: &'a Group<'a>,
    pub options: &'a [&'a OptArg<'a>],
    pub positionals: &'a [&'a PosArg<'a>],
    pub subcommands: &'a [&'a Command<'a>],
    pub subcommand_aliases: &'a [&'a CommandAlias<'a>],
    pub run: CommandRun,
}

pub type CommandRun = for<'a> fn(&Context<'a>) -> Result<(), ErrorKind<'a>>;

pub trait CommandItem: Display {
    fn group(&self) -> &Group;

    fn params(&self) -> Vec<String> {
        Vec::new()
    }

    fn description(&self) -> Cow<'_, str>;

    fn description_ex(&self) -> &[&str] {
        &[]
    }

    fn enum_items(&self) -> &[EnumItem] {
        &[]
    }

    fn default(&self) -> Option<&str> {
        None
    }

    fn environment(&self) -> Option<&str> {
        None
    }
}

impl Display for Command<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl CommandItem for Command<'_> {
    fn group(&self) -> &Group {
        self.group
    }

    fn description(&self) -> Cow<'_, str> {
        self.description.into()
    }

    fn description_ex(&self) -> &[&str] {
        self.description_ex
    }
}

impl<'a> Command<'a> {
    pub fn option_by_id(&self, id: ArgId) -> Option<&OptArg<'a>> {
        self.options.iter().cloned().find(|&opt| opt.id() == id)
    }

    pub fn option_by_short(&self, name: char) -> Option<&OptArg<'a>> {
        self.options.iter().cloned().find(|&opt| opt.short == Some(name))
    }

    pub fn option_by_long(&self, name: &str) -> Option<&OptArg<'a>> {
        self.options.iter().cloned().find(|&opt| opt.long == Some(name))
    }

    pub fn subcommand(&self, name: &str) -> Option<&Command<'a>> {
        self.subcommands.iter().cloned().find(|&cmd| cmd.name == name)
    }

    pub fn subcommand_alias(&self, name: &str) -> Option<&CommandAlias<'a>> {
        self.subcommand_aliases
            .iter()
            .cloned()
            .find(|&alias| alias.name == name)
    }

    pub fn subcommand_or_alias(&self, name: &str) -> Option<(&Command<'a>, &'a [&'a str])> {
        if let Some(subcommand) = self.subcommand(name) {
            return Some((subcommand, &[]));
        }
        if let Some(alias) = self.subcommand_alias(name) {
            return Some((alias.command, alias.args));
        }
        None
    }

    pub fn groups(&self) -> Vec<&Group> {
        let mut groups: Vec<&Group> = Vec::new();
        collect_groups(self.subcommands, &mut groups);
        collect_groups(self.subcommand_aliases, &mut groups);
        collect_groups(self.positionals, &mut groups);
        collect_groups(self.options, &mut groups);
        groups
    }

    pub fn group_items(&self, group: &Group) -> Vec<&dyn CommandItem> {
        let mut items: Vec<&dyn CommandItem> = Vec::new();
        collect_group_items(self.subcommands, &mut items, group);
        collect_group_items(self.subcommand_aliases, &mut items, group);
        collect_group_items(self.positionals, &mut items, group);
        collect_group_items(self.options, &mut items, group);
        items
    }
}

fn collect_groups<'a, T: CommandItem>(items: &'a [&'a T], target: &mut Vec<&'a Group<'a>>) {
    for &item in items {
        if !target.contains(&item.group()) {
            target.push(item.group());
        }
    }
}

fn collect_group_items<'a, T: CommandItem>(items: &'a [&'a T], target: &mut Vec<&'a dyn CommandItem>, group: &Group) {
    for &item in items {
        if item.group() == group {
            target.push(item);
        }
    }
}
