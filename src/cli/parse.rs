use super::ArgValue;
use super::Runner;
use super::args::Args;
use super::types::Command;
use super::types::OptArg;
use super::types::OptArgKind;
use anyhow::bail;
use bstr::B;
use bstr::ByteSlice;
use std::borrow::Cow;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

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
        let mut binary = arg_iter
            .next()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        'next_arg: while let Some(arg) = arg_iter.next() {
            let opt = if let Some(opt_name) = arg.as_bytes().strip_prefix(B("--")) {
                if let Some(opt) = current_command.find_long_opt(opt_name) {
                    Some(opt)
                } else {
                    bail!("Unknown option: --{}", opt_name.to_str_lossy());
                }
            } else if let Some(opt_name) = arg.as_bytes().strip_prefix(B("-")) {
                if let Some(opt) = current_command.find_short_opt(opt_name[0] as char) {
                    Some(opt)
                } else {
                    bail!("Unknown option: -{}", opt_name.to_str_lossy());
                }
            } else {
                None
            };

            if let Some(opt) = opt {
                match opt.kind {
                    OptArgKind::Flag => {
                        arg_values.set(opt, Box::new(true));
                    }
                    OptArgKind::Value(ArgValue { parse, .. }) => {
                        if let Some(raw_value) = arg_iter.next() {
                            match parse(raw_value.into()) {
                                Ok(value) => {
                                    arg_values.set(opt, value);
                                }
                                Err(err) => {
                                    bail!("Invalid option {opt} value: {err}");
                                }
                            }
                        } else {
                            bail!("Missing value for option {opt}");
                        }
                    }
                }
            } else if let Some(command) = self.find_command(arg.as_bytes()) {
                parent_commands.push(current_command);
                current_command = command;
                binary.push(' ');
                binary.push_str(command.name);
            } else if !current_command.commands.is_empty() {
                bail!("Unkown command {}", arg.to_string_lossy());
            } else {
                for pos in current_command.positionals {
                    if arg_values.has(pos) {
                        continue;
                    }
                    match (pos.value.parse)(Cow::Owned(arg)) {
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
