use super::ArgValue;
use super::Runner;
use super::args::Args;
use super::types::Command;
use super::types::OptArg;
use super::types::OptArgKind;
use anyhow::bail;
use os_str_bytes::OsStrBytesExt;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

impl Command {
    pub fn parse_args(&self) -> anyhow::Result<Runner> {
        self.parse_args_from(std::env::args_os())
    }

    pub fn parse_args_from<T: IntoIterator<Item = OsString>>(
        &self,
        args: T,
    ) -> anyhow::Result<Runner> {
        let mut parent_commands = Vec::new();
        let mut current_command = self;
        let mut arg_iter = args.into_iter();
        let mut arg_values = Args::new();
        let mut allow_options = true;

        let mut binary = arg_iter
            .next()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        'next_arg: while let Some(arg) = arg_iter.next() {
            if allow_options {
                if let Some(opt_name) = arg.strip_prefix("--") {
                    if opt_name.is_empty() {
                        allow_options = false;
                        continue 'next_arg;
                    }

                    if let Some((opt_name, opt_value)) = opt_name.split_once("=") {
                        if let Some(opt) = current_command.find_long_opt(opt_name) {
                            match opt.kind {
                                OptArgKind::Flag => {
                                    bail!("Unexpected value for option {opt}");
                                }
                                OptArgKind::Value(ArgValue { parse, .. }) => {
                                    match parse(opt_value.into()) {
                                        Ok(value) => {
                                            arg_values.set_long(opt, value);
                                            continue 'next_arg;
                                        }
                                        Err(err) => {
                                            bail!("Invalid value for option {opt}: {err}");
                                        }
                                    }
                                }
                            }
                        } else {
                            bail!("Unknown option --{}", opt_name.to_string_lossy());
                        }
                    }

                    if let Some(opt) = current_command.find_long_opt(opt_name) {
                        match opt.kind {
                            OptArgKind::Flag => {
                                arg_values.set_long(opt, Box::new(true));
                                continue 'next_arg;
                            }
                            OptArgKind::Value(ArgValue { parse, .. }) => {
                                if let Some(opt_value) = arg_iter.next() {
                                    match parse(opt_value.into()) {
                                        Ok(value) => {
                                            arg_values.set_long(opt, value);
                                            continue 'next_arg;
                                        }
                                        Err(err) => {
                                            bail!("Invalid value for option {opt}: {err}");
                                        }
                                    }
                                } else {
                                    bail!("Missing value for option {opt}");
                                }
                            }
                        }
                    } else {
                        bail!("Unknown option --{}", opt_name.to_string_lossy());
                    }
                }

                if let Some(opt_chars) = arg.strip_prefix("-") {
                    if !opt_chars.is_empty() {
                        let mut value_pos = 0;

                        for (invalid, opt_char_chunk) in opt_chars.utf8_chunks() {
                            if !invalid.as_os_str().is_empty() {
                                bail!("Unknown option -{}", invalid.as_os_str().to_string_lossy());
                            }

                            for opt_char in opt_char_chunk.chars() {
                                value_pos += opt_char.len_utf8();

                                if let Some(opt) = current_command.find_short_opt(opt_char) {
                                    match opt.kind {
                                        OptArgKind::Flag => {
                                            arg_values.set(opt, Box::new(true));
                                        }
                                        OptArgKind::Value(ArgValue { parse, .. }) => {
                                            let (_, opt_value) = opt_chars.split_at(value_pos);

                                            let opt_value: Cow<OsStr> = if opt_value.is_empty() {
                                                match arg_iter.next() {
                                                    Some(opt_value) => opt_value.into(),
                                                    None => bail!("Missing value for option {opt}"),
                                                }
                                            } else {
                                                opt_value.into()
                                            };

                                            match parse(opt_value) {
                                                Ok(value) => {
                                                    arg_values.set(opt, value);
                                                    continue 'next_arg;
                                                }
                                                Err(err) => {
                                                    bail!("Invalid value for option {opt}: {err}");
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    bail!("Unknown option -{opt_char}");
                                }
                            }
                        }

                        continue 'next_arg;
                    }
                }
            }

            if let Some(command) = self.find_command(&arg) {
                parent_commands.push(current_command);
                current_command = command;
                binary.push(' ');
                binary.push_str(command.name);
                continue 'next_arg;
            }

            if !current_command.commands.is_empty() {
                bail!("Unkown command {}", arg.to_string_lossy());
            }

            'next_pos: for pos in current_command.positionals {
                if arg_values.has(pos) {
                    continue 'next_pos;
                }
                match (pos.value.parse)(arg.into()) {
                    Ok(value) => {
                        arg_values.set(pos, value);
                        continue 'next_arg;
                    }
                    Err(err) => {
                        bail!("Invalid argument {} value: {}", pos, err)
                    }
                }
            }

            bail!("Unexpected argument: {}", arg.to_string_lossy());
        }

        Ok(Runner {
            binary,
            command: current_command,
            parents: parent_commands,
            args: arg_values,
        })
    }

    fn find_short_opt(&self, name: char) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.short == Some(name))
    }

    fn find_long_opt(&self, name: &OsStr) -> Option<&OptArg> {
        self.options
            .iter()
            .find(|opt| opt.long.map(OsStr::new) == Some(name))
    }

    fn find_command(&self, name: &OsStr) -> Option<&Command> {
        self.commands
            .iter()
            .find(|cmd| OsStr::new(cmd.name) == name)
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::Command;
    use crate::cli::CommandBuilder;
    use crate::cli::Flag;
    use crate::cli::FlagBuilder;
    use crate::cli::Opt;
    use crate::cli::OptBuilder;
    use crate::cli::Pos;
    use crate::cli::PosBuilder;
    use claims::*;
    use rstest::rstest;
    use std::ffi::OsString;

    const FLAG: Flag = FlagBuilder::new("flag").short('f').long("flag").done();
    const OPT: Opt<i32> = OptBuilder::new("opt").short('o').long("option").done();
    const POS: Pos<String> = PosBuilder::new("pos").name("pos").done();
    const SUB_COMMAND: Command = CommandBuilder::new().name("sub").done();

    #[test]
    fn command() {
        let command = CommandBuilder::new().done();
        let runner = assert_ok!(command.parse_args_from(make_args(&[])));

        assert_eq!(runner.binary, "bin");
        assert_eq!(runner.command, &command);
        assert_eq!(runner.parents, Vec::<&Command>::new());
    }

    #[test]
    fn sub_command() {
        let command = CommandBuilder::new().commands(&[SUB_COMMAND]).done();
        let runner = assert_ok!(command.parse_args_from(make_args(&["sub"])));

        assert_eq!(runner.binary, "bin sub");
        assert_eq!(runner.command, &SUB_COMMAND);
        assert_eq!(runner.parents, &[&command]);
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

        let runner = assert_ok!(command.parse_args_from(make_args(args)));
        assert_eq!(runner.args.get(&FLAG), flag);
        assert_eq!(runner.args.get(&OPT), opt);
        assert_eq!(runner.args.get(&POS), pos);
    }

    #[rstest]
    #[case(&["--flag=value"], "Unexpected value for option -f, --flag")]
    #[case(&["-o"], "Missing value for option -o, --option")]
    #[case(&["-o", "x"], "Invalid value for option -o, --option: invalid digit found in string")]
    #[case(&["--option"], "Missing value for option -o, --option")]
    #[case(&["--option", "x"], "Invalid value for option -o, --option: invalid digit found in string")]
    #[case(&["--option=x"], "Invalid value for option -o, --option: invalid digit found in string")]
    #[case(&["-x"], "Unknown option -x")]
    #[case(&["--xtra"], "Unknown option --xtra")]
    #[case(&["--xtra=x"], "Unknown option --xtra")]
    #[case(&["abc", "def"], "Unexpected argument: def")]
    fn command_args_err(#[case] args: &[&str], #[case] err_msg: &str) {
        let command = CommandBuilder::new()
            .options(&[FLAG.arg, OPT.arg])
            .positionals(&[POS.arg])
            .done();

        let err = assert_err!(command.parse_args_from(make_args(args)));
        assert_eq!(err.to_string(), err_msg);
    }

    fn make_args(args: &[&str]) -> Vec<OsString> {
        let mut args: Vec<OsString> = args.iter().map(OsString::from).collect();
        args.insert(0, "bin".into());
        args
    }
}
