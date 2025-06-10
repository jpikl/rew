use super::COMMAND_PARAM;
use super::CommandItem;
use super::Context;
use super::OptArg;
use super::PosArg;
use crate::cli::ERROR_END;
use crate::cli::ERROR_START;
use crate::cli::QUOTE_END;
use crate::cli::QUOTE_START;
use std::ffi::OsString;
use std::fmt::Display;
use std::io::Write;
use std::process::exit;

// Box error implementation to keep the error object on stack small
#[derive(Debug)]
pub struct Error<'a>(Box<ErrorInner<'a>>);

#[derive(Debug)]
struct ErrorInner<'a> {
    context: Context<'a>,
    kind: ErrorKind<'a>,
}

impl<'a> Error<'a> {
    pub fn new(context: Context<'a>, kind: ErrorKind<'a>) -> Self {
        Self(Box::new(ErrorInner { context, kind }))
    }
}

impl Error<'_> {
    pub fn exit(&self) -> ! {
        let kind = &self.0.kind;

        if kind.is_broken_pipe() {
            exit(0);
        }

        let _ = self.print(&mut anstream::stderr().lock());

        if kind.is_invalid_usage() {
            exit(2);
        }

        exit(1);
    }

    fn print(&self, out: &mut dyn Write) -> std::io::Result<()> {
        let kind = &self.0.kind;
        let context = &self.0.context;

        writeln!(out, "{ERROR_START}{}{ERROR_END}: {kind}", context.calls)?;

        let mut source = kind.source();
        while let Some(cause) = source {
            writeln!(out, "Caused by: {cause}")?;
            source = cause.source();
        }

        for opt in context.commands.current().options {
            if let Some(err_hint) = opt.err_hint {
                err_hint(context, kind, out)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for Error<'_> {}

impl Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.kind)
    }
}

#[derive(Debug)]
pub enum ErrorKind<'a> {
    UnknownShortOption(OsString),
    UnknownLongOption(OsString),
    UnknownSubcommand(OsString),
    MissingOptionValue(&'a OptArg<'a>),
    InvalidOptionValue(&'a OptArg<'a>, OsString, Box<dyn std::error::Error>),
    InvalidArgumentValue(&'a PosArg<'a>, OsString, Box<dyn std::error::Error>),
    InvalidEnvironmentValue(&'a str, OsString, Box<dyn std::error::Error>),
    UnexpectedOptionValue(&'a OptArg<'a>, OsString),
    UnexpectedArgument(OsString),
    MissingArgument(&'a PosArg<'a>),
    MissingSubcommand,
    MutuallyExclusiveOptions(Vec<&'a OptArg<'a>>),
    InvalidUsage(String),
    RuntimeError(Box<dyn std::error::Error>),
}

impl ErrorKind<'_> {
    pub fn msg(msg: impl Into<String>) -> Self {
        SimpleError::new(msg).into()
    }

    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RuntimeError(err) => err.source(),
            _ => None,
        }
    }

    pub fn downcast_ref<T: std::error::Error + 'static>(&self) -> Option<&T> {
        match self {
            Self::RuntimeError(err) => err.downcast_ref(),
            _ => None,
        }
    }

    pub fn is_broken_pipe(&self) -> bool {
        match self.downcast_ref::<std::io::Error>() {
            Some(io_err) => io_err.kind() == std::io::ErrorKind::BrokenPipe,
            None => false,
        }
    }

    pub fn is_invalid_usage(&self) -> bool {
        !matches!(self, Self::RuntimeError(_))
    }
}

impl<E: std::error::Error + 'static> From<E> for ErrorKind<'_> {
    fn from(err: E) -> Self {
        Self::RuntimeError(Box::new(err))
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

                write!(f, " are mutually exclusive")
            }
            Self::InvalidUsage(msg) => msg.fmt(f),
            Self::RuntimeError(err) => err.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct SimpleError(String);

impl SimpleError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl std::error::Error for SimpleError {}

impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
