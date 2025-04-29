use super::Command;
use super::OptArg;
use super::PosArg;
use crate::format::BOLD;
use crate::format::BOLD_RED;
use crate::format::Colorizer;
use crate::format::RED;
use crate::format::RESET;
use crate::format::YELLOW;
use anstream::eprintln;
use std::ffi::OsString;
use std::fmt::Display;
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

#[derive(Debug)]
pub struct Error<'a> {
    pub command: &'a Command<'a>,
    pub call_chain: CallChain,
    pub kind: ErrorKind<'a>,
}

impl std::error::Error for Error<'_> {}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error<'_> {
    pub fn exit(self) -> ! {
        match self.kind {
            ErrorKind::RuntimeError(ref err) if is_broken_pipe(err) => {
                exit(0);
            }
            ErrorKind::RuntimeError(ref err) => {
                self.print_runtime_error(err);
                exit(1);
            }
            ref err => {
                self.print_invalid_usage(err);
                exit(2)
            }
        }
    }

    fn print_runtime_error(&self, err: &anyhow::Error) {
        self.print_prefix();
        eprintln!("{BOLD_RED}Error:{RESET} {err}");

        for cause in err.chain().skip(1) {
            self.print_prefix();
            eprintln!("{RED}└─>{RESET} {cause}");
        }
    }

    fn print_invalid_usage(&self, err: &ErrorKind) {
        self.print_prefix();
        eprintln!("{BOLD_RED}Invalid usage:{RESET} {}", Colorizer(err));

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

    fn print_prefix(&self) {
        eprint!("{BOLD}{}:{RESET} ", self.call_chain);
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

#[derive(Debug)]
pub enum ErrorKind<'a> {
    UnkownShortOption(OsString),
    UnkownLongOption(OsString),
    UnkownSubcommand(OsString),
    MissingOptionValue(&'a OptArg<'a>),
    InvalidOptionValue(&'a OptArg<'a>, OsString, anyhow::Error),
    InvalidArgumentValue(&'a PosArg<'a>, OsString, anyhow::Error),
    UnexpectedOptionValue(&'a OptArg<'a>, OsString),
    UnexpectedArgument(OsString),
    MissingArgument(&'a PosArg<'a>),
    MissingSubcommand,
    MutuallyExclusiveOptions(&'a [&'a OptArg<'a>]),
    RuntimeError(anyhow::Error),
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for ErrorKind<'_> {
    fn from(err: E) -> Self {
        Self::RuntimeError(err.into())
    }
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnkownShortOption(name) => write!(f, "Unknown option `-{}`", name.to_string_lossy()),
            Self::UnkownLongOption(name) => write!(f, "Unknown option `--{}`", name.to_string_lossy()),
            Self::UnkownSubcommand(name) => write!(f, "Unknown subcommand `{}`", name.to_string_lossy()),
            Self::MissingOptionValue(opt) => write!(f, "Missing value for option `{opt}`"),
            Self::InvalidOptionValue(opt, val, err) => {
                write!(f, "Invalid value `{}` for option `{opt}`: {err}", val.to_string_lossy())
            }
            Self::InvalidArgumentValue(arg, val, err) => write!(
                f,
                "Invalid value `{}` for argument `{arg}`: {err}",
                val.to_string_lossy()
            ),
            Self::UnexpectedOptionValue(opt, val) => {
                write!(f, "Unexpected value `{}` for option `{opt}`", val.to_string_lossy())
            }
            Self::UnexpectedArgument(arg) => write!(f, "Unexpected argument `{}`", arg.to_string_lossy()),
            Self::MissingArgument(arg) => write!(f, "Missing argument `{arg}`"),
            Self::MissingSubcommand => write!(f, "Missing subcommand"),
            Self::MutuallyExclusiveOptions(opts) => {
                write!(f, "Options ")?;

                for (i, opt) in opts.iter().enumerate() {
                    if i > 0 {
                        if i < opts.len() - 1 {
                            write!(f, ", ")?;
                        } else {
                            write!(f, " and ")?;
                        }
                    }
                    write!(f, "`{}`", opt)?;
                }

                write!(f, " are mutualy exclusive")
            }
            Self::RuntimeError(err) => err.fmt(f),
        }
    }
}
