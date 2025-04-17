use super::ArgValue;
use super::Error;
use super::ErrorKind;
use super::PosArg;
use super::Runner;
use super::args::Args;
use super::types::Command;
use super::types::OptArg;
use super::types::OptArgKind;
use os_str_bytes::OsStrBytesExt;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::result::Result;

pub struct Parser<'a, I: Iterator> {
    command: &'a Command,
    call_chain: Vec<OsString>,
    values: Args,
    allow_options: bool,
    args: I,
}

impl<'a, I: Iterator<Item = OsString>> Parser<'a, I> {
    pub fn new<T: IntoIterator<IntoIter = I>>(command: &'a Command, args: T) -> Self {
        Self {
            command,
            call_chain: Vec::new(),
            values: Args::new(),
            allow_options: true,
            args: args.into_iter(),
        }
    }

    pub fn parse(mut self) -> Result<Runner<'a>, Error<'a>> {
        if let Some(arg) = self.args.next() {
            self.call_chain.push(arg);
        }

        while let Some(arg) = self.args.next() {
            self.parse_arg(arg)?;
        }

        Ok(Runner {
            command: self.command,
            call_chain: self.call_chain,
            args: self.values,
        })
    }

    fn parse_arg(&mut self, arg: OsString) -> Result<(), Error<'a>> {
        if self.allow_options {
            if let Some(name) = arg.strip_prefix("--") {
                if !name.is_empty() {
                    return self.parse_long_opt(name);
                } else {
                    self.allow_options = false;
                    return Ok(());
                }
            } else if let Some(name) = arg.strip_prefix("-") {
                if !name.is_empty() {
                    return self.parse_short_opts(name);
                }
            }
        }

        if !self.command.subcommands.is_empty() {
            if let Some(subcommand) = self.command.find_subcommand(&arg) {
                self.command = subcommand;
                self.call_chain.push(arg);
                return Ok(());
            } else {
                return Err(self.err(ErrorKind::UnkownSubcommand(arg)));
            }
        }

        if let Some(pos) = self.command.find_unset_pos(&self.values) {
            let err_value = arg.clone(); // For possible error

            match (pos.value.parse)(arg.into()) {
                Ok(value) => {
                    self.values.set(pos, value);
                    return Ok(());
                }
                Err(err) => {
                    return Err(self.err(ErrorKind::InvalidArgumentValue(pos, err_value, err)));
                }
            }
        }

        Err(self.err(ErrorKind::UnexpectedArgument(arg)))
    }

    fn parse_long_opt(&mut self, name: &OsStr) -> Result<(), Error<'a>> {
        let (name, value) = match name.split_once("=") {
            Some((name, value)) => (name, Some(value)),
            None => (name, None),
        };
        if let Some(opt) = self.command.find_long_opt(name) {
            match opt.kind {
                OptArgKind::Flag => {
                    if let Some(value) = value {
                        Err(self.err(ErrorKind::UnexpectedOptionValue(opt, value.into())))
                    } else {
                        self.values.set_long(opt, Box::new(true));
                        Ok(())
                    }
                }
                OptArgKind::Value(ArgValue { parse, .. }) => {
                    let value = match value {
                        Some(value) => Some(value.into()),
                        None => self.args.next().map(Cow::Owned),
                    };

                    if let Some(value) = value {
                        let err_value = value.clone(); // For possible error

                        match parse(value) {
                            Ok(value) => {
                                self.values.set_long(opt, value);
                                Ok(())
                            }
                            Err(err) => Err(self.err(ErrorKind::InvalidOptionValue(
                                opt,
                                err_value.into(),
                                err,
                            ))),
                        }
                    } else {
                        Err(self.err(ErrorKind::MissingOptionValue(opt)))
                    }
                }
            }
        } else {
            Err(self.err(ErrorKind::UnkownLongOption(name.into())))
        }
    }

    fn parse_short_opts(&mut self, name: &OsStr) -> Result<(), Error<'a>> {
        let mut value_pos = 0;

        for (invalid, chunk) in name.utf8_chunks() {
            if !invalid.as_os_str().is_empty() {
                return Err(self.err(ErrorKind::UnkownShortOption(invalid.into())));
            }

            for char in chunk.chars() {
                value_pos += char.len_utf8();
                let (_, value) = name.split_at(value_pos);
                if self.parse_short_opt(char, value)? {
                    break;
                }
            }
        }

        Ok(())
    }

    fn parse_short_opt(&mut self, char: char, value: &OsStr) -> Result<bool, Error<'a>> {
        if let Some(opt) = self.command.find_short_opt(char) {
            match opt.kind {
                OptArgKind::Flag => {
                    self.values.set(opt, Box::new(true));
                    Ok(false)
                }
                OptArgKind::Value(ArgValue { parse, .. }) => {
                    let value: Cow<OsStr> = if value.is_empty() {
                        if let Some(value) = self.args.next() {
                            value.into()
                        } else {
                            return Err(self.err(ErrorKind::MissingOptionValue(opt)));
                        }
                    } else {
                        value.into()
                    };

                    let err_value = value.clone(); // For possible error

                    match parse(value) {
                        Ok(value) => {
                            self.values.set(opt, value);
                            Ok(true)
                        }
                        Err(err) => {
                            Err(self.err(ErrorKind::InvalidOptionValue(opt, err_value.into(), err)))
                        }
                    }
                }
            }
        } else {
            Err(self.err(ErrorKind::UnkownShortOption(char.to_string().into())))
        }
    }

    fn err(&self, kind: ErrorKind<'a>) -> Error<'a> {
        Error {
            call_chain: self.call_chain.clone(),
            kind,
        }
    }
}

impl Command {
    fn find_short_opt(&self, name: char) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.short == Some(name))
    }

    fn find_long_opt(&self, name: &OsStr) -> Option<&OptArg> {
        self.options
            .iter()
            .find(|opt| opt.long.map(OsStr::new) == Some(name))
    }

    fn find_subcommand(&self, name: &OsStr) -> Option<&Command> {
        self.subcommands
            .iter()
            .find(|cmd| OsStr::new(cmd.name) == name)
    }

    fn find_unset_pos(&self, args: &Args) -> Option<&PosArg> {
        self.positionals.iter().find(|pos| !args.has(*pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Command;
    use crate::cli::CommandBuilder;
    use crate::cli::Flag;
    use crate::cli::FlagBuilder;
    use crate::cli::Opt;
    use crate::cli::OptBuilder;
    use crate::cli::Pos;
    use crate::cli::PosBuilder;
    use anstream::StripStream;
    use bstr::ByteSlice;
    use claims::*;
    use rstest::rstest;
    use std::ffi::OsString;

    const FLAG: Flag = FlagBuilder::new("flag").short('f').long("flag").done();
    const OPT: Opt<i32> = OptBuilder::new("opt").short('o').long("option").done();
    const POS: Pos<String> = PosBuilder::new("pos").name("pos").done();
    const SUBCOMMAND: Command = CommandBuilder::new().name("sub").done();

    #[test]
    fn command() {
        let command = CommandBuilder::new().done();
        let parser = Parser::new(&command, make_args(&[]));
        let runner = assert_ok!(parser.parse());

        assert_eq!(runner.command, &command);
        assert_eq!(runner.call_chain, vec![OsString::from("bin")]);
    }

    #[test]
    fn subcommand() {
        let command = CommandBuilder::new().subcommands(&[SUBCOMMAND]).done();
        let parser = Parser::new(&command, make_args(&["sub"]));
        let runner = assert_ok!(parser.parse());

        assert_eq!(runner.command, &SUBCOMMAND);
        assert_eq!(
            runner.call_chain,
            vec![OsString::from("bin"), OsString::from("sub")]
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
    fn command_args(
        #[case] args: &[&str],
        #[case] flag: bool,
        #[case] opt: i32,
        #[case] pos: &str,
    ) {
        let command = CommandBuilder::new()
            .options(&[FLAG.arg, OPT.arg])
            .positionals(&[POS.arg])
            .done();

        let parser = Parser::new(&command, make_args(args));
        let runner = assert_ok!(parser.parse());

        assert_eq!(runner.args.get(&FLAG), flag);
        assert_eq!(runner.args.get(&OPT), opt);
        assert_eq!(runner.args.get(&POS), pos);
    }

    #[rstest]
    #[case(&["--flag=x"], "bin: Unexpected value 'x' for option '-f, --flag'")]
    #[case(&["-o"], "bin: Missing value for option '-o, --option'")]
    #[case(&["-o", "x"], "bin: Invalid value 'x' for option '-o, --option': invalid digit found in string")]
    #[case(&["--option"], "bin: Missing value for option '-o, --option'")]
    #[case(&["--option", "x"], "bin: Invalid value 'x' for option '-o, --option': invalid digit found in string")]
    #[case(&["--option=x"], "bin: Invalid value 'x' for option '-o, --option': invalid digit found in string")]
    #[case(&["-x"], "bin: Unknown option '-x'")]
    #[case(&["--xtra"], "bin: Unknown option '--xtra'")]
    #[case(&["--xtra=x"], "bin: Unknown option '--xtra'")]
    #[case(&["arg", "x"], "bin: Unexpected argument 'x'")]
    fn command_args_err(#[case] args: &[&str], #[case] err_msg: &str) {
        let command = CommandBuilder::new()
            .options(&[FLAG.arg, OPT.arg])
            .positionals(&[POS.arg])
            .done();

        let parser = Parser::new(&command, make_args(args));
        let err = assert_err!(parser.parse());

        use std::io::Write;
        let mut output = StripStream::new(Vec::new());
        assert_ok!(write!(&mut output, "{err}"));
        assert_eq!(output.into_inner().to_str_lossy(), err_msg);
    }

    fn make_args(args: &[&str]) -> Vec<OsString> {
        let mut args: Vec<OsString> = args.iter().map(OsString::from).collect();
        args.insert(0, "bin".into());
        args
    }
}
