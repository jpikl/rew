use crate::cli::COMMAND_ALIASES;
use crate::cli::Command;
use crate::cli::CommandItem;
use crate::cli::Group;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, PartialEq)]
pub struct CommandAlias<'a> {
    pub name: &'a str,
    pub command: &'a Command<'a>,
    pub args: &'a [&'a str],
}

impl<'a> Command<'a> {
    pub const fn alias(&'a self, name: &'a str) -> CommandAlias<'a> {
        CommandAlias {
            name,
            command: self,
            args: &[],
        }
    }

    pub const fn alias_with_args(&'a self, name: &'a str, args: &'a [&'a str]) -> CommandAlias<'a> {
        CommandAlias {
            name,
            command: self,
            args,
        }
    }
}

impl Display for CommandAlias<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl CommandItem for CommandAlias<'_> {
    fn group(&self) -> &Group {
        &COMMAND_ALIASES
    }

    fn description(&self) -> Cow<'_, str> {
        let mut desc = String::from("Alias for `");
        desc.push_str(self.command.name);

        for &arg in self.args {
            desc.push(' ');
            desc.push_str(arg);
        }

        desc.push('`');
        desc.into()
    }
}
