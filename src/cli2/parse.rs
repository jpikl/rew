use crate::cli2::Arg;
use crate::cli2::ArgSetter;
use crate::cli2::Command;
use crate::cli2::Id;
use os_str_bytes::OsStrBytesExt;
use std::ffi::OsStr;
use std::ffi::OsString;

pub enum RawArgs {
    ArgsOs(std::env::ArgsOs),
    DynIter(Box<dyn Iterator<Item = OsString>>),
}

impl RawArgs {
    pub fn from_env() -> Self {
        Self::ArgsOs(std::env::args_os())
    }

    pub fn next(&mut self) -> Option<RawArg> {
        match self {
            Self::ArgsOs(iter) => iter.next().map(RawArg),
            Self::DynIter(iter) => iter.next().map(RawArg),
        }
    }
}

impl<T: IntoIterator<Item = I>, I: Into<OsString>> From<T> for RawArgs {
    fn from(items: T) -> Self {
        let vec: Vec<OsString> = items.into_iter().map(Into::into).collect();
        Self::DynIter(Box::new(vec.into_iter()))
    }
}

pub struct RawArg(OsString);

impl RawArg {
    fn parse(self) -> ParsedArg {
        if let Some(value) = self.0.strip_prefix("--") {
            if value.is_empty() {
                return ParsedArg::OptTerminator;
            }
            if let Some((key, value)) = value.split_once("=") {
                return ParsedArg::LongOptVal(key.into(), value.into());
            }
            return ParsedArg::LongOpt(value.into());
        }
        if let Some(value) = self.0.strip_prefix("-") {
            if !value.is_empty() {
                return ParsedArg::ShortOpts(ShortOpts::new(value));
            }
        }
        ParsedArg::Val(self.0)
    }
}

#[derive(Debug)]
pub enum ParsedArg {
    ShortOpts(ShortOpts),
    LongOpt(OsString),
    LongOptVal(OsString, OsString),
    OptTerminator,
    Val(OsString),
}

#[derive(Debug)]
pub struct ShortOpts(OsString);

impl ShortOpts {
    fn new(value: &OsStr) -> Self {
        Self(value.into())
    }

    fn split_first(&self) -> Option<Result<(char, &OsStr), &OsStr>> {
        if let Some((bad, ok)) = self.0.utf8_chunks().next() {
            let bad = bad.as_os_str();
            if !bad.is_empty() {
                return Some(Err(bad));
            }
            if let Some(ch) = ok.chars().next() {
                let (_, remainder) = self.0.split_at(ch.len_utf8());
                return Some(Ok((ch, remainder)));
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnknownShortOption(OsString),
    UnknownLongOption(OsString),
    UnknownSubcommand(OsString),
    MissingOptionValue(&'static dyn Arg),
    InvalidOptionValue(&'static dyn Arg, OsString, Box<dyn std::error::Error>),
    InvalidArgumentValue(&'static dyn Arg, OsString, Box<dyn std::error::Error>),
    InvalidEnvironmentValue(&'static str, OsString, Box<dyn std::error::Error>),
    UnexpectedOptionValue(&'static dyn Arg, OsString),
    UnexpectedArgument(OsString),
    MissingArgument(&'static dyn Arg),
    MissingSubcommand,
    UnhandledMatch(Id, Option<OsString>),
}

pub struct Parser {
    args: Option<RawArgs>,
    next_short_opts: Option<ShortOpts>,
    command: &'static Command,
    command_chain: Vec<&'static Command>,
    call_chain: Vec<OsString>,
    allow_options: bool,
    positional_index: usize,
}

impl Parser {
    pub fn new(command: &'static Command, args: RawArgs) -> Self {
        Self {
            args: Some(args),
            next_short_opts: None,
            command,
            command_chain: Vec::new(),
            call_chain: Vec::new(),
            allow_options: true,
            positional_index: 0,
        }
    }

    pub fn process(&mut self, setters: &mut [&mut dyn ArgSetter]) -> Result<(), ParseError> {
        while let Some((id, value)) = self.next_match()? {
            if let Some(setter) = setters.iter_mut().find(|s| s.id() == id) {
                if let Some(value) = value {
                    setter.set_from(value)?;
                } else {
                    setter.set();
                }
            } else {
                self.handle_match(id, value)?;
            }
        }
        Ok(())
    }

    pub fn handle_match(&self, id: Id, value: Option<OsString>) -> Result<(), ParseError> {
        Err(ParseError::UnhandledMatch(id, value))
    }

    fn next_arg(&mut self) -> Option<RawArg> {
        match &mut self.args {
            Some(args) => args.next(),
            None => None,
        }
    }

    pub fn next_match(&mut self) -> Result<Option<(Id, Option<OsString>)>, ParseError> {
        if let Some(short_opts) = self.next_short_opts.take() {
            return self.next_match_from_short_opts(short_opts);
        }
        let Some(arg) = self.next_arg() else {
            return Ok(None);
        };
        if self.call_chain.is_empty() {
            self.call_chain.push(arg.0);
            return self.next_match();
        }
        if self.allow_options {
            return self.next_match_from_parsed_arg(arg.parse());
        }
        self.next_match_from_positional(arg.0)
    }

    fn next_match_from_parsed_arg(
        &mut self,
        parsed_arg: ParsedArg,
    ) -> Result<Option<(Id, Option<OsString>)>, ParseError> {
        match parsed_arg {
            // "-abcde"
            ParsedArg::ShortOpts(opts) => self.next_match_from_short_opts(opts),
            // "--long"
            ParsedArg::LongOpt(name) => self.next_match_from_long_opt(name),
            // "--long=value"
            ParsedArg::LongOptVal(name, value) => self.next_match_from_long_opt_value(name, value),
            // "--"
            ParsedArg::OptTerminator => {
                self.allow_options = false;
                self.next_match()
            }
            // "value"
            ParsedArg::Val(value) => {
                self.allow_options = false;
                self.next_match_from_positional(value)
            }
        }
    }

    fn next_match_from_short_opts(&mut self, opts: ShortOpts) -> Result<Option<(Id, Option<OsString>)>, ParseError> {
        match opts.split_first() {
            Some(Ok((name, value))) => {
                if let Some(arg) = self
                    .command
                    .args
                    .iter()
                    .cloned()
                    .find(|a| a.short().is_some_and(|arg_name| arg_name == name))
                {
                    if arg.value().is_some() {
                        if value.is_empty() {
                            if let Some(next_arg) = self.next_arg() {
                                Ok(Some((arg.id(), Some(next_arg.0))))
                            } else {
                                Err(ParseError::MissingOptionValue(arg))
                            }
                        } else {
                            Ok(Some((arg.id(), Some(value.into()))))
                        }
                    } else {
                        if !value.is_empty() {
                            self.next_short_opts.replace(ShortOpts::new(value));
                        }
                        Ok(Some((arg.id(), None)))
                    }
                } else {
                    Err(ParseError::UnknownShortOption(name.to_string().into()))
                }
            }
            Some(Err(invalid)) => Err(ParseError::UnknownShortOption(invalid.to_owned())),
            None => self.next_match(),
        }
    }

    fn next_match_from_long_opt(&mut self, name: OsString) -> Result<Option<(Id, Option<OsString>)>, ParseError> {
        if let Some(arg) = self
            .command
            .args
            .iter()
            .cloned()
            .find(|a| a.long().is_some_and(|arg_name| arg_name == name))
        {
            if arg.value().is_some() {
                if let Some(next_arg) = self.next_arg() {
                    Ok(Some((arg.id(), Some(next_arg.0))))
                } else {
                    Err(ParseError::MissingOptionValue(arg))
                }
            } else {
                Ok(Some((arg.id(), None)))
            }
        } else {
            Err(ParseError::UnknownLongOption(name))
        }
    }

    fn next_match_from_long_opt_value(
        &self,
        name: OsString,
        value: OsString,
    ) -> Result<Option<(Id, Option<OsString>)>, ParseError> {
        if let Some(arg) = self
            .command
            .args
            .iter()
            .cloned()
            .find(|a| a.long().is_some_and(|arg_name| arg_name == name))
        {
            if arg.value().is_some() {
                Ok(Some((arg.id(), Some(value))))
            } else {
                Err(ParseError::UnexpectedOptionValue(arg, value))
            }
        } else {
            Err(ParseError::UnknownLongOption(name))
        }
    }

    fn next_match_from_positional(&mut self, value: OsString) -> Result<Option<(Id, Option<OsString>)>, ParseError> {
        if let Some(arg) = self
            .command
            .args
            .iter()
            .cloned()
            .filter(|&arg| arg.is_positional())
            .nth(self.positional_index)
        {
            if !arg.multple() {
                self.positional_index += 1;
            }
            Ok(Some((arg.id(), Some(value))))
        } else {
            Err(ParseError::UnexpectedArgument(value))
        }
    }
}
