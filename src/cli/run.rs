use super::ArgSource;
use super::Args;
use super::Command;
use super::Error;
use super::ErrorKind;
use std::ffi::OsString;

#[derive(Debug)]
pub struct Context<'a> {
    pub calls: CallChain,
    pub commands: CommandChain<'a>,
    pub args: Args,
}

impl<'a> Context<'a> {
    pub fn run(self) {
        if let Err(err) = self.try_run() {
            err.exit();
        }
    }

    pub fn try_run(self) -> Result<(), Error<'a>> {
        let command = self.commands.current();

        for usage in self.args.usages() {
            if let ArgSource::Positional = usage.source {
                continue;
            }
            if let Some(run) = command.option_by_id(&usage.arg_id).and_then(|opt| opt.run) {
                return run(&self, usage).map_err(|err| Error::new(self, err));
            }
        }

        if !command.subcommands.is_empty() {
            return Err(Error::new(self, ErrorKind::MissingSubcommand));
        }

        for &pos in command.positionals {
            if pos.required && !self.args.has(pos) {
                return Err(Error::new(self, ErrorKind::MissingArgument(pos)));
            }
        }

        (command.run)(&self).map_err(|err| Error::new(self, err))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallChain(pub Vec<OsString>);

impl std::fmt::Display for CallChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, str) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", str.to_string_lossy())?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandChain<'a>(pub Vec<&'a Command<'a>>);

impl<'a> CommandChain<'a> {
    pub fn current(&self) -> &Command<'a> {
        self.0.last().expect("command chain is empty")
    }
}

impl std::fmt::Display for CommandChain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, command) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", command.name)?;
        }
        Ok(())
    }
}
