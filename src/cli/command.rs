use super::Context;
use super::EnumItem;
use super::ErrorKind;
use super::Group;
use super::OptArg;
use super::PosArg;
use std::fmt::Display;

pub const OPTIONS_PARAM: &str = "[OPTIONS]";
pub const COMMAND_PARAM: &str = "<COMMAND>";

#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub description_ex: &'a [&'a str],
    pub version: Option<&'a str>,
    pub group: &'a Group<'a>,
    pub options: &'a [OptArg<'a>],
    pub positionals: &'a [PosArg<'a>],
    pub subcommands: &'a [Command<'a>],
    pub run: CommandRun,
}

pub type CommandRun = for<'a> fn(&Context<'a>) -> Result<(), ErrorKind<'a>>;

pub trait CommandItem {
    fn group(&self) -> &Group;
    fn params(&self) -> Vec<String>;
    fn description(&self) -> &str;
    fn description_ex(&self) -> &[&str];
    fn enum_items(&self) -> &[EnumItem];
    fn default(&self) -> Option<&str>;
    fn environment(&self) -> Option<&str>;
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

    fn params(&self) -> Vec<String> {
        let mut params: Vec<String> = Vec::new();

        if !self.options.is_empty() {
            params.push(OPTIONS_PARAM.into());
        }

        for positional in self.positionals {
            params.push(positional.to_string());
        }

        if !self.subcommands.is_empty() {
            params.push(COMMAND_PARAM.into());
        }

        params
    }

    fn description(&self) -> &str {
        self.description
    }

    fn description_ex(&self) -> &[&str] {
        self.description_ex
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

impl<'a> Command<'a> {
    pub fn option_by_id(&self, id: &str) -> Option<&OptArg<'a>> {
        self.options.iter().find(|opt| opt.id == id)
    }

    pub fn option_by_short(&self, name: char) -> Option<&OptArg<'a>> {
        self.options.iter().find(|opt| opt.short == Some(name))
    }

    pub fn option_by_long(&self, name: &str) -> Option<&OptArg<'a>> {
        self.options.iter().find(|opt| opt.long == Some(name))
    }

    pub fn subcommand(&self, name: &str) -> Option<&Command<'a>> {
        self.subcommands.iter().find(|cmd| cmd.name == name)
    }

    pub fn grouped_options(&self) -> Vec<(&Group, Vec<&OptArg<'a>>)> {
        group_items(self.options)
    }

    pub fn grouped_positionals(&self) -> Vec<(&Group, Vec<&PosArg<'a>>)> {
        group_items(self.positionals)
    }

    pub fn grouped_subcommands(&self) -> Vec<(&Group, Vec<&Command<'a>>)> {
        group_items(self.subcommands)
    }
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
