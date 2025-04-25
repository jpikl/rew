use super::Command;
use super::OptArg;
use super::PosArg;
use crate::colors::BOLD;
use crate::colors::BOLD_RED;
use crate::colors::Colorizer;
use crate::colors::RED;
use crate::colors::RESET;
use crate::colors::YELLOW;
use anstream::eprintln;
use derive_more::Display;
use derive_more::Error;
use std::ffi::OsString;
use std::process::exit;

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

#[derive(Debug, Error, Display)]
#[display("{kind}")]
pub struct Error<'a> {
    pub command: &'a Command,
    pub call_chain: CallChain,
    pub kind: ErrorKind<'a>,
}

impl Error<'_> {
    fn print_prefix(&self) {
        eprint!("{BOLD}{}:{RESET} ", self.call_chain);
    }

    fn prin_invalid_usage(&self) {
        self.print_prefix();
        eprintln!("{BOLD_RED}Invalid usage:{RESET} {}", Colorizer(&self.kind));

        if let Some(help) = self.command.help_option() {
            if let Some(short) = help.short {
                self.print_prefix();
                eprintln!(
                    "For more information, try '{YELLOW}{} -{}{RESET}'.",
                    self.call_chain, short
                );
            } else if let Some(long) = help.long {
                self.print_prefix();
                eprintln!(
                    "For more information, try '{YELLOW}{} --{}{RESET}'.",
                    self.call_chain, long
                );
            }
        }
    }

    fn print_run_error(&self, err: &anyhow::Error) {
        self.print_prefix();
        eprintln!("{BOLD_RED}Error:{RESET} {err}");

        for cause in err.chain().skip(1) {
            self.print_prefix();
            eprintln!("{RED}└─>{RESET} {cause}");
        }
    }
}

impl Error<'_> {
    pub fn exit(self) -> ! {
        match self.kind {
            ErrorKind::RunError(ref err) if is_broken_pipe(err) => {
                exit(0);
            }
            ErrorKind::RunError(ref err) => {
                self.print_run_error(err);
                exit(1);
            }
            _ => {
                self.prin_invalid_usage();
                exit(2)
            }
        }
    }
}

fn is_broken_pipe(err: &anyhow::Error) -> bool {
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        if io_err.kind() == std::io::ErrorKind::BrokenPipe {
            return true;
        }
    }
    false
}

#[derive(Debug, Display)]
pub enum ErrorKind<'a> {
    #[display("Unknown option `-{}`", _0.to_string_lossy())]
    UnkownShortOption(OsString),
    #[display("Unknown option `--{}`", _0.to_string_lossy())]
    UnkownLongOption(OsString),
    #[display("Unknown subcommand `{}`", _0.to_string_lossy())]
    UnkownSubcommand(OsString),
    #[display("Missing value for option `{_0}`")]
    MissingOptionValue(&'a OptArg),
    #[display("Invalid value `{}` for option `{_0}`: {_2}", _1.to_string_lossy())]
    InvalidOptionValue(&'a OptArg, OsString, anyhow::Error),
    #[display("Invalid value `{}` for argument `{_0}`: {_2}", _1.to_string_lossy())]
    InvalidArgumentValue(&'a PosArg, OsString, anyhow::Error),
    #[display("Unexpected value `{}` for option `{_0}`", _1.to_string_lossy())]
    UnexpectedOptionValue(&'a OptArg, OsString),
    #[display("Unexpected argument `{}`", _0.to_string_lossy())]
    UnexpectedArgument(OsString),
    #[display("Missing argument `{_0}`")]
    MissingArgument(&'a PosArg),
    #[display("Missing subcommand")]
    MissingSubcommand,
    RunError(anyhow::Error),
}
