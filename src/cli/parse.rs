use super::args::Arg;
use super::args::Args;
use super::types::Command;
use super::types::OptArg;
use super::types::OptKind;
use anyhow::format_err;
use bstr::B;
use bstr::ByteSlice;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

impl Command {
    pub fn parse_args(&self) -> anyhow::Result<Args> {
        self.parse_args_from(std::env::args_os())
    }

    pub fn parse_args_from<T: IntoIterator<Item = OsString>>(
        &self,
        args: T,
    ) -> anyhow::Result<Args> {
        let mut iter = args.into_iter().skip(1);
        let mut parsed_args = Args::new();

        while let Some(arg) = iter.next() {
            let opt = if let Some(opt_name) = arg.as_bytes().strip_prefix(B("--")) {
                if let Some(opt) = self.find_long_opt(opt_name) {
                    Some(opt)
                } else {
                    return Err(format_err!("Unknown option: {}", opt_name.to_str_lossy()));
                }
            } else if let Some(opt_name) = arg.as_bytes().strip_prefix(B("-")) {
                if let Some(opt) = self.find_short_opt(opt_name[0] as char) {
                    Some(opt)
                } else {
                    return Err(format_err!("Unknown option: {}", opt_name.to_str_lossy()));
                }
            } else {
                None
            };

            if let Some(opt) = opt {
                match opt.kind {
                    OptKind::Flag => {
                        parsed_args.set(opt, Box::new(true));
                    }
                    OptKind::Value { parse, .. } => {
                        if let Some(raw_value) = iter.next() {
                            match parse(raw_value.into()) {
                                Ok(value) => {
                                    parsed_args.set(opt, value);
                                }
                                Err(err) => {
                                    return Err(format_err!(
                                        "Invalid option {} value: {}",
                                        opt.key(),
                                        err
                                    ));
                                }
                            }
                        } else {
                            return Err(format_err!("Missing value for option: {}", opt.key()));
                        }
                    }
                }
            } else {
                unimplemented!("handle positional arguments");
            }
        }

        Ok(parsed_args)
    }

    fn find_short_opt(&self, name: char) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.short == name)
    }

    fn find_long_opt(&self, name: &[u8]) -> Option<&OptArg> {
        self.options.iter().find(|opt| opt.long.as_bytes() == name)
    }
}
