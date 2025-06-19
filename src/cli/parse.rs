use super::Arg;
use super::ArgUsage;
use super::Args;
use super::CallChain;
use super::Command;
use super::CommandChain;
use super::Context;
use super::Error;
use super::ErrorKind;
use super::ParseValue;
use super::Value;
use super::ValueSource;
use os_str_bytes::OsStrBytesExt;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::result::Result;
use std::vec::IntoIter;

impl<'a> Command<'a> {
    pub fn parse_args(&'a self) -> Context<'a> {
        match self.try_parse_args() {
            Ok(context) => context,
            Err(err) => err.exit(),
        }
    }

    pub fn try_parse_args(&'a self) -> Result<Context<'a>, Error<'a>> {
        self.try_parse_args_from(std::env::args_os())
    }

    pub fn try_parse_args_from(
        &'a self,
        args: impl IntoIterator<IntoIter = impl Iterator<Item = OsString>>,
    ) -> Result<Context<'a>, Error<'a>> {
        let mut parser = Parser::new(self, args);

        match parser.parse() {
            Ok(()) => Ok(parser.into()),
            Err(err) => Err(Error::new(parser.into(), err)),
        }
    }
}

impl<'a, I> From<Parser<'a, I>> for Context<'a> {
    fn from(parser: Parser<'a, I>) -> Self {
        Self {
            calls: CallChain(parser.calls),
            commands: CommandChain(parser.commands),
            args: Args::new(parser.usages),
        }
    }
}

struct Parser<'a, I> {
    args: I,
    injected_args: IntoIter<OsString>,
    command: &'a Command<'a>,
    commands: Vec<&'a Command<'a>>,
    calls: Vec<OsString>,
    usages: Vec<ArgUsage<'a>>,
    allow_options: bool,
    positional_index: usize,
}

impl<'a, I: Iterator<Item = OsString>> Parser<'a, I> {
    fn new<T: IntoIterator<IntoIter = I>>(command: &'a Command<'a>, args: T) -> Self {
        Self {
            args: args.into_iter(),
            injected_args: IntoIter::default(),
            usages: Vec::new(),
            command,
            commands: Vec::new(),
            calls: Vec::new(),
            allow_options: true,
            positional_index: 0,
        }
    }

    fn parse(&mut self) -> Result<(), ErrorKind<'a>> {
        self.commands.push(self.command);

        if let Some(arg) = self.next_arg() {
            self.calls.push(arg);
        } else {
            self.calls.push(self.command.name.into());
        }

        while let Some(arg) = self.next_arg() {
            self.parse_arg(arg)?;
        }

        self.parse_env()
    }

    fn parse_arg(&mut self, arg: OsString) -> Result<(), ErrorKind<'a>> {
        if self.allow_options {
            if let Some(name) = arg.strip_prefix("--") {
                return if !name.is_empty() {
                    self.parse_long_option(name)
                } else {
                    self.allow_options = false;
                    Ok(())
                };
            } else if let Some(name) = arg.strip_prefix("-") {
                if !name.is_empty() {
                    return self.parse_short_options(name);
                }
            }
        }

        if !self.command.subcommands.is_empty() {
            let Some(name) = arg.to_str() else {
                return Err(ErrorKind::UnknownSubcommand(arg));
            };

            let Some((subcommand, alias_args)) = self.command.subcommand_or_alias(name) else {
                return Err(ErrorKind::UnknownSubcommand(arg));
            };

            self.command = subcommand;
            self.commands.push(subcommand);
            self.calls.push(subcommand.name.into()); // Arg might be alias, we need real command name
            self.positional_index = 0;

            if !alias_args.is_empty() {
                self.injected_args = alias_args
                    .iter()
                    .map(OsString::from)
                    .collect::<Vec<OsString>>()
                    .into_iter();
            }

            return Ok(());
        }

        let Some(&pos) = self.command.positionals.get(self.positional_index) else {
            return Err(ErrorKind::UnexpectedArgument(arg));
        };

        if !pos.multiple {
            self.positional_index += 1;
        }

        match (pos.value.parse)(arg.into()) {
            Ok(value) => {
                self.usages.push(ArgUsage::new(pos, ValueSource::Positional, value));
                Ok(())
            }
            Err((value, err)) => Err(ErrorKind::InvalidArgumentValue(pos, value, err)),
        }
    }

    fn parse_long_option(&mut self, name: &OsStr) -> Result<(), ErrorKind<'a>> {
        let (name, value) = match name.split_once("=") {
            Some((name, value)) => (name, Some(value)),
            None => (name, None),
        };

        let Some(name) = name.to_str() else {
            return Err(ErrorKind::UnknownLongOption(name.into()));
        };

        let Some(opt) = self.command.option_by_long(name) else {
            return Err(ErrorKind::UnknownLongOption(name.into()));
        };

        match opt.value {
            None => {
                if let Some(value) = value {
                    return Err(ErrorKind::UnexpectedOptionValue(opt, value.into()));
                }
                self.usages
                    .push(ArgUsage::new(opt, ValueSource::LongOption, Box::new(true)));
                Ok(())
            }
            Some(Value { parse, .. }) => {
                let value = match value {
                    Some(value) => Some(value.into()),
                    None => self.next_arg().map(Cow::Owned),
                };
                let Some(value) = value else {
                    return Err(ErrorKind::MissingOptionValue(opt));
                };
                match parse(value) {
                    Ok(value) => {
                        self.usages.push(ArgUsage::new(opt, ValueSource::LongOption, value));
                        Ok(())
                    }
                    Err((value, err)) => Err(ErrorKind::InvalidOptionValue(opt, value, err)),
                }
            }
        }
    }

    fn parse_short_options(&mut self, chars: &OsStr) -> Result<(), ErrorKind<'a>> {
        let mut suffix_pos = 0;

        for (invalid, chunk) in chars.utf8_chunks() {
            if !invalid.as_os_str().is_empty() {
                return Err(ErrorKind::UnknownShortOption(invalid.into()));
            }
            for char in chunk.chars() {
                suffix_pos += char.len_utf8();
                let (_, suffix) = chars.split_at(suffix_pos);
                if self.parse_short_option(char, suffix)? {
                    break;
                }
            }
        }

        Ok(())
    }

    fn parse_short_option(&mut self, char: char, suffix: &OsStr) -> Result<bool, ErrorKind<'a>> {
        let Some(opt) = self.command.option_by_short(char) else {
            return Err(ErrorKind::UnknownShortOption(char.to_string().into()));
        };

        match opt.value {
            None => {
                self.usages
                    .push(ArgUsage::new(opt, ValueSource::ShortOption, Box::new(true)));
                Ok(false)
            }
            Some(Value { parse, .. }) => {
                let value: Cow<OsStr> = if suffix.is_empty() {
                    let Some(next_arg) = self.next_arg() else {
                        return Err(ErrorKind::MissingOptionValue(opt));
                    };
                    next_arg.into()
                } else {
                    suffix.into()
                };
                match parse(value) {
                    Ok(value) => {
                        self.usages.push(ArgUsage::new(opt, ValueSource::ShortOption, value));
                        Ok(true)
                    }
                    Err((value, err)) => Err(ErrorKind::InvalidOptionValue(opt, value, err)),
                }
            }
        }
    }

    fn parse_env(&mut self) -> Result<(), ErrorKind<'a>> {
        for &opt in self.command.options {
            if self.usages.iter().any(|usage| usage.id == opt.id()) {
                continue;
            }
            let Some(key) = opt.environment else {
                continue;
            };
            let Some(value) = std::env::var_os(key) else {
                continue;
            };
            let parse = match opt.value {
                None => bool::parse_value,
                Some(Value { parse, .. }) => parse,
            };
            match parse(value.into()) {
                Ok(value) => {
                    self.usages.push(ArgUsage::new(opt, ValueSource::Environment, value));
                }
                Err((value, err)) => {
                    return Err(ErrorKind::InvalidEnvironmentValue(key, value, err));
                }
            }
        }

        Ok(())
    }

    fn next_arg(&mut self) -> Option<OsString> {
        self.injected_args.next().or_else(|| self.args.next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::build::CommandBuilder;
    use crate::cli::build::FlagBuilder;
    use crate::cli::build::OptBuilder;
    use crate::cli::build::PosBuilder;
    use crate::cli::option::Flag;
    use crate::cli::option::Opt;
    use crate::cli::positional::Pos;
    use crate::utils::strip_colors;
    use claims::*;
    use rstest::rstest;
    use std::ffi::OsString;

    const FLAG: Flag = FlagBuilder::new()
        .short('f')
        .long("flag")
        .environment("REW_FLAG")
        .done();

    const OPT: Opt<i32> = OptBuilder::new()
        .short('o')
        .long("option")
        .environment("REW_OPTION")
        .done();

    const POS: Pos<String> = PosBuilder::new().name("pos").done();
    const SUBCOMMAND: Command = CommandBuilder::new().name("sub").done();

    #[test]
    fn command() {
        let command = CommandBuilder::new().done();
        let context = assert_ok!(command.try_parse_args_from(make_args(&[])));

        assert_eq!(context.commands, CommandChain(vec![&command]));
        assert_eq!(context.calls, CallChain(vec![OsString::from("bin")]));
    }

    #[test]
    fn subcommand() {
        let command = CommandBuilder::new().subcommands(&[&SUBCOMMAND]).done();
        let context = assert_ok!(command.try_parse_args_from(make_args(&["sub"])));

        assert_eq!(context.commands, CommandChain(vec![&command, &SUBCOMMAND]));
        assert_eq!(
            context.calls,
            CallChain(vec![OsString::from("bin"), OsString::from("sub")])
        );
    }

    #[rstest]
    #[case(&[], false, 0, "")]
    // Separate flag/option/positional
    #[case(&["-f"], true, 0, "")]
    #[case(&["--flag"], true, 0, "")]
    #[case(&["-o123"], false, 123, "")]
    #[case(&["-o", "123"], false, 123, "")]
    #[case(&["--option", "123"], false, 123, "")]
    #[case(&["--option=123"], false, 123, "")]
    #[case(&["abc"], false, 0, "abc")]
    #[case(&["-"], false, 0, "-")]
    // Skip option parsing after --
    #[case(&["--", "-f"], false, 0, "-f")]
    #[case(&["--", "--flag"], false, 0, "--flag")]
    #[case(&["--", "-o"], false, 0, "-o")]
    #[case(&["--", "-o123"], false, 0, "-o123")]
    #[case(&["--", "--option"], false, 0, "--option")]
    #[case(&["--", "--option=123"], false, 0, "--option=123")]
    #[case(&["--", "abc"], false, 0, "abc")]
    // Combinations
    #[case(&["-ffo123", "abc"], true, 123, "abc")]
    #[case(&["-ffo", "123", "abc"], true, 123, "abc")]
    #[case(&["--flag", "--option", "123", "abc"], true, 123, "abc")]
    #[case(&["-ffo123", "-o456", "abc"], true, 456, "abc")]
    fn command_args(#[case] args: &[&str], #[case] flag: bool, #[case] opt: i32, #[case] pos: &str) {
        let command = CommandBuilder::new()
            .options(&[&FLAG.arg, &OPT.arg])
            .positionals(&[&POS.arg])
            .done();

        let context = assert_ok!(command.try_parse_args_from(make_args(args)));
        assert_eq!(context.args.get(&FLAG), flag);
        assert_eq!(context.args.get(&OPT), opt);
        assert_eq!(context.args.get(&POS), pos);
    }

    #[rstest]
    #[case(&[], "REW_FLAG", None, false, 0)]
    #[case(&[], "REW_FLAG", Some("true"), true, 0)]
    #[case(&["-f"], "REW_FLAG", None, true, 0)]
    #[case(&["-f"], "REW_FLAG", Some("true"), true, 0)]
    #[case(&["-f"], "REW_FLAG", Some("x"), true, 0)]
    #[case(&[], "REW_OPTION", None, false, 0)]
    #[case(&[], "REW_OPTION", Some("456"), false, 456)]
    #[case(&["-o123"], "REW_OPTION", None, false, 123)]
    #[case(&["-o123"], "REW_OPTION", Some("456"), false, 123)]
    #[case(&["-o123"], "REW_OPTION", Some("x"), false, 123)]
    fn command_env(
        #[case] args: &[&str],
        #[case] env_key: &str,
        #[case] env_value: Option<&str>,
        #[case] flag: bool,
        #[case] opt: i32,
    ) {
        temp_env::with_var(env_key, env_value, || {
            let command = CommandBuilder::new().options(&[&FLAG.arg, &OPT.arg]).done();

            let context = assert_ok!(command.try_parse_args_from(make_args(args)));
            assert_eq!(context.args.get(&FLAG), flag);
            assert_eq!(context.args.get(&OPT), opt);
        });
    }

    #[rstest]
    #[case(&["--flag=x"], "Option '-f, --flag' got unexpected value 'x'")]
    #[case(&["-o"], "Option '-o, --option' requires value '<VALUE>'")]
    #[case(&["-o", "x"], "Option '-o, --option' got invalid value 'x': invalid digit found in string")]
    #[case(&["--option"], "Option '-o, --option' requires value '<VALUE>'")]
    #[case(&["--option", "x"], "Option '-o, --option' got invalid value 'x': invalid digit found in string")]
    #[case(&["--option=x"], "Option '-o, --option' got invalid value 'x': invalid digit found in string")]
    #[case(&["-x"], "Unknown option '-x'")]
    #[case(&["--extra"], "Unknown option '--extra'")]
    #[case(&["--extra=x"], "Unknown option '--extra'")]
    #[case(&["arg", "x"], "Unexpected argument 'x'")]
    fn command_args_err(#[case] args: &[&str], #[case] err_msg: &str) {
        let command = CommandBuilder::new()
            .options(&[&FLAG.arg, &OPT.arg])
            .positionals(&[&POS.arg])
            .done();

        let err = assert_err!(command.try_parse_args_from(make_args(args)));
        assert_eq!(strip_colors(&err.to_string()), err_msg);
    }

    #[rstest]
    #[case(
        "REW_FLAG",
        "",
        "Environment variable 'REW_FLAG' got invalid value '': provided string was not `true` or `false`"
    )]
    #[case(
        "REW_FLAG",
        "x",
        "Environment variable 'REW_FLAG' got invalid value 'x': provided string was not `true` or `false`"
    )]
    #[case(
        "REW_OPTION",
        "",
        "Environment variable 'REW_OPTION' got invalid value '': cannot parse integer from empty string"
    )]
    #[case(
        "REW_OPTION",
        "x",
        "Environment variable 'REW_OPTION' got invalid value 'x': invalid digit found in string"
    )]
    fn command_env_err(#[case] env_key: &str, #[case] env_value: &str, #[case] err_msg: &str) {
        temp_env::with_var(env_key, Some(env_value), || {
            let command = CommandBuilder::new().options(&[&FLAG.arg, &OPT.arg]).done();
            let err = assert_err!(command.try_parse_args_from(make_args(&[])));
            assert_eq!(strip_colors(&err.to_string()), err_msg);
        });
    }

    fn make_args(args: &[&str]) -> Vec<OsString> {
        let mut args: Vec<OsString> = args.iter().map(OsString::from).collect();
        args.insert(0, "bin".into());
        args
    }
}
