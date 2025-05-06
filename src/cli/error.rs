use super::COMMAND_PARAM;
use super::Command;
use super::CommandItem;
use super::OptArg;
use super::PosArg;
use anstream::eprintln;
use std::ffi::OsString;
use std::fmt::Display;
use std::process::exit;

pub const ERROR_START: &str = "\x1b[1;31m";
pub const ERROR_END: &str = "\x1b[0m";

pub const QUOTE_START: &str = "'\x1b[1;33m";
pub const QUOTE_END: &str = "\x1b[0m'";

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
        self.print_error(err);

        for cause in err.chain().skip(1) {
            eprintln!("Caused by: {cause}");
        }
    }

    fn print_invalid_usage(&self, err: &ErrorKind) {
        self.print_error(err);

        if let Some(help) = self.command.help_option() {
            if let Some(short) = help.short {
                eprintln!(
                    "Try {QUOTE_START}{} -{}{QUOTE_END} for more information.",
                    self.call_chain, short
                );
            } else if let Some(long) = help.long {
                eprintln!(
                    "Try {QUOTE_START}{} --{}{QUOTE_END} for more information.",
                    self.call_chain, long
                );
            }
        }
    }

    fn print_error(&self, err: &impl Display) {
        eprintln!("{ERROR_START}{}{ERROR_END}: {err}", self.call_chain);
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
            Self::UnkownShortOption(name) => {
                write!(f, "Unknown option {QUOTE_START}-{}{QUOTE_END}", name.to_string_lossy())
            }
            Self::UnkownLongOption(name) => {
                write!(f, "Unknown option {QUOTE_START}--{}{QUOTE_END}", name.to_string_lossy())
            }
            Self::UnkownSubcommand(name) => {
                write!(
                    f,
                    "Unknown subcommand {QUOTE_START}{}{QUOTE_END}",
                    name.to_string_lossy()
                )
            }
            Self::MissingOptionValue(opt) => write!(
                f,
                "Option {QUOTE_START}{opt}{QUOTE_END} requires value {QUOTE_START}{}{QUOTE_END}",
                opt.params().join(" ")
            ),
            Self::InvalidOptionValue(opt, val, err) => {
                write!(
                    f,
                    "Option {QUOTE_START}{opt}{QUOTE_END} got invalid value {QUOTE_START}{}{QUOTE_END}: {err}",
                    val.to_string_lossy()
                )
            }
            Self::InvalidArgumentValue(arg, val, err) => write!(
                f,
                "Argument {QUOTE_START}{arg}{QUOTE_END} got invalid value {QUOTE_START}{}{QUOTE_END}: {err}",
                val.to_string_lossy()
            ),
            Self::UnexpectedOptionValue(opt, val) => {
                write!(
                    f,
                    "Option {QUOTE_START}{opt}{QUOTE_END} got unexpected value {QUOTE_START}{}{QUOTE_END}",
                    val.to_string_lossy()
                )
            }
            Self::UnexpectedArgument(arg) => {
                write!(
                    f,
                    "Unexpected argument {QUOTE_START}{}{QUOTE_END}",
                    arg.to_string_lossy()
                )
            }
            Self::MissingArgument(arg) => write!(f, "Missing required argument {QUOTE_START}{arg}{QUOTE_END}"),
            Self::MissingSubcommand => write!(f, "Missing required argument {QUOTE_START}{COMMAND_PARAM}{QUOTE_END}"),
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
                    write!(f, "{QUOTE_START}{}{QUOTE_END}", opt)?;
                }

                write!(f, " are mutualy exclusive")
            }
            Self::RuntimeError(err) => err.fmt(f),
        }
    }
}
