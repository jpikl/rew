use super::Arg;
use super::ArgId;
use super::ArgUsage;
use super::CommandItem;
use super::Context;
use super::EnumItem;
use super::ErrorKind;
use super::Group;
use super::TypedArg;
use super::Value;
use std::any::Any;
use std::fmt::Display;
use std::io::Write;

pub type Flag<'a> = TypedArg<OptArg<'a>, bool>;
pub type Opt<'a, T> = TypedArg<OptArg<'a>, T>;

#[derive(Debug, PartialEq)]
pub struct OptArg<'a> {
    pub short: Option<char>,
    pub long: Option<&'a str>,
    pub description: &'a str,
    pub description_ex: &'a [&'a str],
    pub group: &'a Group<'a>,
    pub environment: Option<&'a str>,
    pub value: Option<Value<'a>>,
    pub run: Option<OptRun>,
    pub err_hint: Option<OptErrHint>,
}

pub type OptRun = for<'a> fn(&Context<'a>, &ArgUsage) -> Result<(), ErrorKind<'a>>;
pub type OptErrHint = for<'a> fn(&Context<'a>, err: &ErrorKind<'a>, out: &mut dyn Write) -> std::io::Result<()>;

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

impl Arg for OptArg<'_> {
    fn id(&self) -> ArgId {
        let short = self.short.unwrap_or_default();
        let long = self.long.unwrap_or_default();
        if short as u32 == 0 && long.is_empty() {
            ArgId::Undefined
        } else {
            ArgId::OptName(short, long)
        }
    }

    fn default_value(&self) -> Option<Box<dyn Any>> {
        self.value.as_ref().and_then(|value| value.default())
    }
}

impl CommandItem for OptArg<'_> {
    fn group(&self) -> &Group {
        self.group
    }

    fn params(&self) -> Vec<String> {
        let mut params = Vec::new();

        if let Some(value) = &self.value {
            params.push(format!("<{}>", value.name));
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
        self.value.as_ref().map_or(&[], |value| value.enum_items)
    }

    fn default(&self) -> Option<&str> {
        self.value.as_ref().and_then(|value| value.default)
    }

    fn environment(&self) -> Option<&str> {
        self.environment
    }
}
