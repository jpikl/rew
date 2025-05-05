use super::EnumItem;
use super::ErrorKind;
use super::args::Arg;
use super::args::Args;
use super::args::ParseOsArgFn;
use super::args::TypedArg;
use std::any::Any;
use std::ffi::OsStr;
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
    pub run: fn(Args) -> Result<(), ErrorKind<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct OptArg<'a> {
    pub id: &'a str,
    pub short: Option<char>,
    pub long: Option<&'a str>,
    pub description: &'a str,
    pub description_ex: &'a [&'a str],
    pub group: &'a Group<'a>,
    pub environment: Option<&'a str>,
    pub kind: OptArgKind<'a>,
}

#[derive(Debug, PartialEq)]
pub enum OptArgKind<'a> {
    Flag,
    Value(ArgValue<'a>),
}

#[derive(Debug, PartialEq)]
pub struct PosArg<'a> {
    pub id: &'a str,
    pub description: &'a str,
    pub description_ex: &'a [&'a str],
    pub group: &'a Group<'a>,
    pub environment: Option<&'a str>,
    pub value: ArgValue<'a>,
    pub required: bool,
}

#[derive(Debug, PartialEq)]
pub struct ArgValue<'a> {
    pub name: &'a str,
    pub default: Option<&'a str>,
    pub enum_items: &'a [EnumItem<'a>],
    pub parse: ParseOsArgFn,
}

impl Arg for OptArg<'_> {
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

impl Arg for PosArg<'_> {
    fn id(&self) -> &str {
        self.id
    }

    fn default_value(&self) -> Option<Box<dyn Any>> {
        self.value.default()
    }
}

impl ArgValue<'_> {
    pub fn default(&self) -> Option<Box<dyn Any>> {
        self.default
            .map(|value| (self.parse)(OsStr::new(value).into()).expect("unparsable default value"))
    }
}

impl Display for Command<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for OptArg<'_> {
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

impl Display for PosArg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.required {
            write!(f, "<{}>", self.value.name)
        } else {
            write!(f, "[{}]", self.value.name)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Group<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
}

pub const COMMANDS: Group = Group {
    name: "Commands",
    description: None,
};

pub const OPTIONS: Group = Group {
    name: "Options",
    description: None,
};

pub const USAGE_OPTIONS: Group = Group {
    name: "Usage options",
    description: None,
};

pub const ARGUMENTS: Group = Group {
    name: "Arguments",
    description: None,
};

pub trait CommandItem {
    fn group(&self) -> &Group;
    fn params(&self) -> Vec<String>;
    fn description(&self) -> &str;
    fn description_ex(&self) -> &[&str];
    fn enum_items(&self) -> &[EnumItem];
    fn default(&self) -> Option<&str>;
    fn environment(&self) -> Option<&str>;
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

impl CommandItem for OptArg<'_> {
    fn group(&self) -> &Group {
        self.group
    }

    fn params(&self) -> Vec<String> {
        match &self.kind {
            OptArgKind::Flag => Vec::new(),
            OptArgKind::Value(value) => vec![format!("<{}>", value.name)],
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

impl CommandItem for PosArg<'_> {
    fn group(&self) -> &Group {
        self.group
    }

    fn params(&self) -> Vec<String> {
        Vec::new()
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

pub type Flag<'a> = TypedArg<OptArg<'a>, bool>;
pub type Opt<'a, T> = TypedArg<OptArg<'a>, T>;
pub type Pos<'a, T> = TypedArg<PosArg<'a>, T>;
