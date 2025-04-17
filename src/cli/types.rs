use super::EnumItem;
use super::args::Arg;
use super::args::Args;
use super::args::ParseOsArgFn;
use super::args::TypedArg;
use std::any::Any;
use std::ffi::OsStr;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub description_ex: &'static [&'static str],
    pub version: Option<&'static str>,
    pub group: &'static Group,
    pub options: &'static [OptArg],
    pub positionals: &'static [PosArg],
    pub subcommands: &'static [Command],
    pub run: fn(Args) -> anyhow::Result<()>,
}

#[derive(Debug, PartialEq)]
pub struct OptArg {
    pub id: &'static str,
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub description: &'static str,
    pub description_ex: &'static [&'static str],
    pub group: &'static Group,
    pub environment: Option<&'static str>,
    pub kind: OptArgKind,
}

#[derive(Debug, PartialEq)]
pub enum OptArgKind {
    Flag,
    Value(ArgValue),
}

#[derive(Debug, PartialEq)]
pub struct PosArg {
    pub id: &'static str,
    pub description: &'static str,
    pub description_ex: &'static [&'static str],
    pub group: &'static Group,
    pub environment: Option<&'static str>,
    pub value: ArgValue,
    pub required: bool,
}

#[derive(Debug, PartialEq)]
pub struct ArgValue {
    pub name: &'static str,
    pub default: Option<&'static str>,
    pub enum_items: &'static [EnumItem],
    pub parse: ParseOsArgFn,
}

impl Arg for OptArg {
    fn id(&self) -> &str {
        self.id
    }

    fn default_value(&self) -> Option<Box<dyn Any>> {
        match &self.kind {
            OptArgKind::Value(value) => value.default(),
            _ => None,
        }
    }
}

impl Arg for PosArg {
    fn id(&self) -> &str {
        self.id
    }

    fn default_value(&self) -> Option<Box<dyn Any>> {
        self.value.default()
    }
}

impl ArgValue {
    pub fn default(&self) -> Option<Box<dyn Any>> {
        self.default
            .map(|value| (self.parse)(OsStr::new(value).into()).expect("unparsable default value"))
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for OptArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(short) = self.short {
            write!(f, "-{}", short)?;
        }
        if self.short.is_some() && self.long.is_some() {
            write!(f, ", ")?;
        }
        if let Some(long) = self.long {
            write!(f, "--{}", long)?;
        }
        Ok(())
    }
}

impl Display for PosArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.name)
    }
}

#[derive(Debug, PartialEq)]
pub struct Group {
    pub name: &'static str,
    pub description: Option<&'static str>,
}

pub const COMMANDS: Group = Group {
    name: "Commands",
    description: None,
};

pub const OPTIONS: Group = Group {
    name: "Options",
    description: None,
};

pub const ARGUMENTS: Group = Group {
    name: "Arguments",
    description: None,
};

pub trait CommandItem {
    fn group(&self) -> &Group;
    fn usage(&self) -> String;
    fn description(&self) -> &str;
    fn description_ex(&self) -> &[&str];
    fn enum_items(&self) -> &[EnumItem];
    fn default(&self) -> Option<&str>;
    fn environment(&self) -> Option<&str>;
}

impl Command {
    pub fn usage_params(&self) -> String {
        let mut usage = String::new();

        if !self.options.is_empty() {
            usage.push_str(" [OPTIONS]");
        }

        for positional in self.positionals {
            usage.push(' ');
            usage.push_str(&positional.usage());
        }

        if !self.subcommands.is_empty() {
            usage.push_str(" <COMMAND>");
        }

        usage
    }
}

impl CommandItem for Command {
    fn group(&self) -> &Group {
        self.group
    }

    fn usage(&self) -> String {
        self.name.to_string()
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

impl CommandItem for OptArg {
    fn group(&self) -> &Group {
        self.group
    }

    fn usage(&self) -> String {
        match &self.kind {
            OptArgKind::Flag => self.to_string(),
            OptArgKind::Value(value) => format!("{self} <{}>", value.name),
        }
    }

    fn description(&self) -> &str {
        self.description
    }

    fn description_ex(&self) -> &[&str] {
        self.description_ex
    }

    fn enum_items(&self) -> &[EnumItem] {
        match &self.kind {
            OptArgKind::Flag => &[],
            OptArgKind::Value(value) => value.enum_items,
        }
    }

    fn default(&self) -> Option<&str> {
        match &self.kind {
            OptArgKind::Flag => None,
            OptArgKind::Value(value) => value.default,
        }
    }

    fn environment(&self) -> Option<&str> {
        self.environment
    }
}

impl CommandItem for PosArg {
    fn group(&self) -> &Group {
        self.group
    }

    fn usage(&self) -> String {
        if self.required {
            format!("<{self}>")
        } else {
            format!("[{self}]")
        }
    }

    fn description(&self) -> &str {
        self.description
    }

    fn description_ex(&self) -> &[&str] {
        self.description_ex
    }

    fn enum_items(&self) -> &[EnumItem] {
        self.value.enum_items
    }

    fn default(&self) -> Option<&str> {
        self.value.default
    }

    fn environment(&self) -> Option<&str> {
        self.environment
    }
}

pub type Flag = TypedArg<OptArg, bool>;
pub type Opt<T> = TypedArg<OptArg, T>;
pub type Pos<T> = TypedArg<PosArg, T>;
