use super::OptArg;
use super::PosArg;
use crate::colors::BOLD_RED;
use crate::colors::RESET;
use crate::colors::YELLOW;
use derive_more::Display;
use derive_more::Error;
use std::ffi::OsString;

#[derive(Debug, Error, Display)]
#[display("{BOLD_RED}{}{RESET}: {kind}", self.prefix())]
pub struct Error<'a> {
    pub call_chain: Vec<OsString>,
    pub kind: ErrorKind<'a>,
}

impl Error<'_> {
    pub fn prefix(&self) -> String {
        self.call_chain
            .iter()
            .map(|str| str.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn category(&self) -> ErrorCategory {
        match self.kind {
            ErrorKind::RunError(_) => ErrorCategory::RuntimeError,
            _ => ErrorCategory::InvalidUsage,
        }
    }
}

pub enum ErrorCategory {
    InvalidUsage,
    RuntimeError,
}

#[derive(Debug, Display)]
pub enum ErrorKind<'a> {
    #[display("Unknown option '{YELLOW}-{}{RESET}'", _0.to_string_lossy())]
    UnkownShortOption(OsString),
    #[display( "Unknown option '{YELLOW}--{}{RESET}'", _0.to_string_lossy())]
    UnkownLongOption(OsString),
    #[display("Unknown subcommand '{YELLOW}{}{RESET}'", _0.to_string_lossy())]
    UnkownSubcommand(OsString),
    #[display("Missing value for option '{YELLOW}{_0}{RESET}'")]
    MissingOptionValue(&'a OptArg),
    #[display( "Invalid value '{YELLOW}{}{RESET}' for option '{YELLOW}{_0}{RESET}': {_2}", _1.to_string_lossy())]
    InvalidOptionValue(&'a OptArg, OsString, anyhow::Error),
    #[display("Invalid value '{YELLOW}{}{RESET}' for argument '{YELLOW}{_0}{RESET}': {_2}", _1.to_string_lossy())]
    InvalidArgumentValue(&'a PosArg, OsString, anyhow::Error),
    #[display( "Unexpected value '{YELLOW}{}{RESET}' for option '{YELLOW}{_0}{RESET}'", _1.to_string_lossy())]
    UnexpectedOptionValue(&'a OptArg, OsString),
    #[display("Unexpected argument '{YELLOW}{}{RESET}'", _0.to_string_lossy())]
    UnexpectedArgument(OsString),
    #[display("Missing argument '{YELLOW}{_0}{RESET}'")]
    MissingArgument(&'a PosArg),
    #[display("Missing subcommand")]
    MissingSubcommand,
    RunError(anyhow::Error),
}
