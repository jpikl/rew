use super::ArgValue;
use super::Runner;
use super::args::Args;
use super::types::Command;
use super::types::OptArg;
use super::types::OptArgKind;
use anyhow::bail;
use bstr::B;
use bstr::BString;
use bstr::ByteSlice;
use std::ffi::OsString;

fn os_to_bstring(str: OsString) -> BString {
    #[cfg(target_family = "unix")]
    let bytes = {
        use std::os::unix::ffi::OsStringExt;
        str.into_vec()
    };

    #[cfg(not(target_family = "unix"))]
    let bytes = str.to_string_lossy().into_owned().into_bytes();

    bytes.into()
}

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
            let arg = os_to_bstring(arg);

            if allow_options {
                if let Some(opt_name) = arg.strip_prefix(B("--")) {
                    if opt_name.is_empty() {
                        allow_options = false;
                        continue 'next_arg;
                    }

                    if let Some((opt_name, opt_value)) = opt_name.split_once_str("=") {
                        if let Some(opt) = current_command.find_long_opt(opt_name) {
                            match opt.kind {
                                OptArgKind::Flag => {
                                    bail!("Unexpected value for option {opt}");
                                }
                                OptArgKind::Value(ArgValue { parse, .. }) => {
                                    match parse(opt_value.as_bstr().into()) {
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
                            bail!("Unknown option: --{}", opt_name.to_str_lossy());
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
                                    match parse(os_to_bstring(opt_value).into()) {
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
                        bail!("Unknown option: --{}", opt_name.to_str_lossy());
                    }
                }

                if let Some(opt_chars) = arg.strip_prefix(B("-")) {
                    if !opt_chars.is_empty() {
                        let mut opt_chars = opt_chars.chars();

                        while let Some(opt_char) = opt_chars.next() {
                            if let Some(opt) = current_command.find_short_opt(opt_char) {
                                match opt.kind {
                                    OptArgKind::Flag => {
                                        arg_values.set(opt, Box::new(true));
                                    }
                                    OptArgKind::Value(ArgValue { parse, .. }) => {
                                        let mut opt_value = opt_chars.clone().collect::<BString>();

                                        if opt_value.is_empty() {
                                            opt_value = match arg_iter.next() {
                                                Some(opt_value) => os_to_bstring(opt_value),
                                                None => bail!("Missing value for option {opt}"),
                                            };
                                        }

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
                                bail!("Unknown option: -{opt_char}");
                            }
                        }

                        continue 'next_arg;
                    }
                }
            }

            if let Some(command) = self.find_command(arg.as_bytes()) {
                parent_commands.push(current_command);
                current_command = command;
                binary.push(' ');
                binary.push_str(command.name);
                continue 'next_arg;
            }

            if !current_command.commands.is_empty() {
                bail!("Unkown command {arg}");
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

    fn find_long_opt(&self, name: &[u8]) -> Option<&OptArg> {
        self.options
            .iter()
            .find(|opt| opt.long.map(|long| long.as_bytes()) == Some(name))
    }

    fn find_command(&self, name: &[u8]) -> Option<&Command> {
        self.commands.iter().find(|cmd| cmd.name.as_bytes() == name)
    }
}
