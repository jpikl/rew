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
                            bail!("Unknown option: --{}", opt_name.to_string_lossy());
                        }
                    }

                    if let Some(opt) = current_command.find_long_opt(opt_name) {
                        match opt.kind {
                            OptArgKind::Flag => {
                                arg_values.set(opt, Box::new(true));
                                continue 'next_arg;
                            }
                            OptArgKind::Value(ArgValue { parse, .. }) => {
                                if let Some(opt_value) = arg_iter.next() {
                                    match parse(opt_value.into()) {
                                        Ok(value) => {
                                            arg_values.set(opt, value);
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
                        bail!("Unknown option: --{}", opt_name.to_string_lossy());
                    }
                }

                if let Some(opt_chars) = arg.strip_prefix("-") {
                    if !opt_chars.is_empty() {
                        let mut value_pos = 0;

                        for (invalid, opt_char_chunk) in opt_chars.utf8_chunks() {
                            if !invalid.as_os_str().is_empty() {
                                bail!("Unknown option: -{}", invalid.as_os_str().to_string_lossy());
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
                                    bail!("Unknown option: -{opt_char}");
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

            bail!("Unexpected argument: {:?}", arg);
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
