use super::ArgValue;
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
    pub fn parse_args(&self) -> anyhow::Result<(&Command, Args)> {
        self.parse_args_from(std::env::args_os())
    }

    pub fn parse_args_from<T: IntoIterator<Item = OsString>>(
        &self,
        args: T,
    ) -> anyhow::Result<(&Command, Args)> {
        let mut current_cmd = self;
        let mut iter = args.into_iter().skip(1);
        let mut parsed_args = Args::new();

        'next_arg: while let Some(arg) = iter.next() {
            let opt = if let Some(opt_name) = arg.as_bytes().strip_prefix(B("--")) {
                if let Some(opt) = current_cmd.find_long_opt(opt_name) {
                    Some(opt)
                } else {
                    bail!("Unknown option: --{}", opt_name.to_str_lossy());
                }
            } else if let Some(opt_name) = arg.as_bytes().strip_prefix(B("-")) {
                if let Some(opt) = current_cmd.find_short_opt(opt_name[0] as char) {
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
                        parsed_args.set(opt, Box::new(true));
                    }
                    OptArgKind::Value(ArgValue { parse, .. }) => {
                        if let Some(raw_value) = iter.next() {
                            match parse(raw_value.into()) {
                                Ok(value) => {
                                    parsed_args.set(opt, value);
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
            } else if let Some(cmd) = self.find_command(arg.as_bytes()) {
                current_cmd = cmd;
            } else if !current_cmd.commands.is_empty() {
                bail!("Unkown command {}", arg.to_string_lossy());
            } else {
                for pos in current_cmd.positionals {
                    if parsed_args.has(pos) {
                        continue;
                    }
                    match (pos.value.parse)(Cow::Owned(arg)) {
                        Ok(value) => {
                            parsed_args.set(pos, value);
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

        Ok((current_cmd, parsed_args))
    }

    fn find_short_opt(&self, name: char) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.short == name)
    }

    fn find_long_opt(&self, name: &[u8]) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.long.as_bytes() == name)
    }

    fn find_command(&self, name: &[u8]) -> Option<&Command> {
        self.commands.iter().find(|cmd| cmd.name.as_bytes() == name)
    }
}
