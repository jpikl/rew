use super::COMMAND_PARAM;
use super::CommandItem;
use super::Context;
use super::HELP_ID;
use super::OptArg;
use super::PosArg;
use crate::cli::ERROR_END;
use crate::cli::ERROR_START;
use crate::cli::QUOTE_END;
use crate::cli::QUOTE_START;
use anstream::eprintln;
use std::ffi::OsString;
use std::fmt::Display;
use std::process::exit;

#[derive(Debug)]
pub struct Error<'a> {
    pub context: Context<'a>,
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
                self.print_error(err);

                for cause in err.chain().skip(1) {
                    eprintln!("Caused by: {cause}");
                }

                exit(1);
            }
            ref err => {
                self.print_error(err);

                if let Some(help) = self.context.commands.current().option_by_id(HELP_ID) {
                    self.print_help_usage(help);
                }

                exit(2)
            }
        }
    }

    fn print_error(&self, err: &impl Display) {
        eprintln!("{ERROR_START}{}{ERROR_END}: {err}", self.context.calls);
    }

    fn print_help_usage(&self, help: &OptArg<'_>) {
        if let Some(short) = help.short {
            eprintln!(
                "Try {QUOTE_START}{} -{}{QUOTE_END} for more information.",
                self.context.calls, short
            );
        } else if let Some(long) = help.long {
            eprintln!(
                "Try {QUOTE_START}{} --{}{QUOTE_END} for more information.",
                self.context.calls, long
            );
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

#[derive(Debug)]
pub enum ErrorKind<'a> {
    UnknownShortOption(OsString),
    UnknownLongOption(OsString),
    UnknownSubcommand(OsString),
    MissingOptionValue(&'a OptArg<'a>),
    InvalidOptionValue(&'a OptArg<'a>, OsString, anyhow::Error),
    InvalidArgumentValue(&'a PosArg<'a>, OsString, anyhow::Error),
    InvalidEnvironmentValue(&'a str, OsString, anyhow::Error),
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
            Self::UnknownShortOption(name) => {
                write!(f, "Unknown option {QUOTE_START}-{}{QUOTE_END}", name.to_string_lossy())
            }
            Self::UnknownLongOption(name) => {
                write!(f, "Unknown option {QUOTE_START}--{}{QUOTE_END}", name.to_string_lossy())
            }
            Self::UnknownSubcommand(name) => {
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
            Self::InvalidEnvironmentValue(key, val, err) => {
                write!(
                    f,
                    "Environment variable {QUOTE_START}{key}{QUOTE_END} got invalid value {QUOTE_START}{}{QUOTE_END}: {err}",
                    val.to_string_lossy()
                )
            }
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
