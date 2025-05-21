use super::Arg;
use super::CommandItem;
use super::EnumItem;
use super::Group;
use super::TypedArg;
use super::Value;
use std::any::Any;
use std::fmt::Display;

pub type Pos<'a, T> = TypedArg<PosArg<'a>, T>;

#[derive(Debug, PartialEq)]
pub struct PosArg<'a> {
    pub id: &'a str,
    pub description: &'a str,
    pub description_ex: &'a [&'a str],
    pub group: &'a Group<'a>,
    pub environment: Option<&'a str>,
    pub value: Value<'a>,
    pub required: bool,
    pub multiple: bool,
}

impl Display for PosArg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.required {
            write!(f, "<{}>", self.value.name)?;
        } else {
            write!(f, "[{}]", self.value.name)?;
        }
        if self.multiple {
            write!(f, "...")?;
        }
        Ok(())
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
