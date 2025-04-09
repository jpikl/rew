use super::EnumItem;
use super::args::Arg;
use super::args::Args;
use super::args::ParseOsArgFn;
use super::args::TypedArg;
use std::any::Any;
use std::ffi::OsStr;
use std::fmt::Display;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub description_ex: &'static [&'static str],
    pub version: &'static str,
    pub group: &'static Group,
    pub options: &'static [OptArg],
    pub positionals: &'static [PosArg],
    pub commands: &'static [Command],
    pub run: fn(Args) -> anyhow::Result<()>,
}

pub struct OptArg {
    pub id: &'static str,
    pub short: char,
    pub long: &'static str,
    pub description: &'static str,
    pub description_ex: &'static [&'static str],
    pub group: &'static Group,
    pub environment: Option<&'static str>,
    pub kind: OptArgKind,
}

pub enum OptArgKind {
    Flag,
    Value(ArgValue),
}

pub struct PosArg {
    pub id: &'static str,
    pub description: &'static str,
    pub description_ex: &'static [&'static str],
    pub group: &'static Group,
    pub environment: Option<&'static str>,
    pub value: ArgValue,
}

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

impl Display for OptArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.short != '\0' {
            write!(f, "-{}", self.short)?;
        }
        if self.short != '\0' && !self.long.is_empty() {
            write!(f, ", ")?;
        }
        if !self.long.is_empty() {
            write!(f, "--{}", self.long)?;
        }
        Ok(())
    }
}

impl Display for PosArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.name)
    }
}

pub struct Group {
    pub name: &'static str,
    pub description: &'static str,
}

pub const COMMANDS: Group = Group {
    name: "Commands",
    description: "",
};

pub const OPTIONS: Group = Group {
    name: "Options",
    description: "",
};

pub const ARGUMENTS: Group = Group {
    name: "Arguments",
    description: "",
};

pub type Flag = TypedArg<OptArg, bool>;
pub type Opt<T> = TypedArg<OptArg, T>;
pub type Pos<T> = TypedArg<PosArg, T>;
