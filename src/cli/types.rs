use super::args::Arg;
use super::args::Args;
use super::args::ParseArgFn;
use super::args::TypedArg;
use std::borrow::Cow;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub group: Option<&'static CommandGroup>,
    pub options: &'static [OptArg],
    pub positionals: &'static [PosArg],
    pub commands: &'static [Command],
    pub run: fn(Args) -> anyhow::Result<()>,
}

pub struct CommandGroup {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct OptArg {
    pub short: char,
    pub long: &'static str,
    pub description: &'static str,
    pub environment: Option<&'static str>,
    pub kind: OptKind,
}

pub enum OptKind {
    Flag,
    Value {
        name: &'static str,
        default: Option<&'static str>,
        parse: ParseArgFn,
    },
}

pub struct PosArg {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
    pub environment: Option<&'static str>,
    pub parse: ParseArgFn,
}

pub type Flag = TypedArg<OptArg, bool>;
pub type Opt<T> = TypedArg<OptArg, T>;
pub type Pos<T> = TypedArg<PosArg, T>;

impl Arg for OptArg {
    fn key(&self) -> Cow<str> {
        if self.short != '\0' {
            Cow::Owned(self.short.to_string())
        } else {
            Cow::Borrowed(self.long)
        }
    }
}

impl Arg for PosArg {
    fn key(&self) -> Cow<str> {
        Cow::Borrowed(self.name)
    }
}
