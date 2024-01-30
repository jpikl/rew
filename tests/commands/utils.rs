use assert_cmd::assert::Assert;
use assert_cmd::crate_name;
use assert_cmd::prelude::*;
use assert_cmd::Command;
use claims::assert_ok;
use std::env;
use std::ffi::OsString;
use std::process;
use std::time::Duration;

#[derive(Clone)]
pub struct Tc {
    bin: OsString,
    args: Vec<OsString>,
    stdin: Option<Vec<u8>>,
}

impl Tc {
    #[must_use]
    pub fn cmd(cmd: &str) -> Self {
        Self::new(Self::bin()).arg(cmd)
    }

    #[must_use]
    pub fn shell(cmd: &str) -> Self {
        let bin = env::var_os("SHELL").unwrap_or("sh".into());
        let cmd = cmd.replace("%bin%", &Self::bin().to_string_lossy());
        Self::new(bin).arg("-c").arg(cmd)
    }

    fn bin() -> OsString {
        let command = process::Command::cargo_bin(crate_name!());
        assert_ok!(command).get_program().to_owned()
    }

    fn new<T: Into<OsString>>(bin: T) -> Self {
        Self {
            bin: bin.into(),
            args: Vec::new(),
            stdin: None,
        }
    }

    #[must_use]
    pub fn arg<T: Into<OsString>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn stdin<T: Into<Vec<u8>>>(mut self, stdin: T) -> Self {
        self.stdin.replace(stdin.into());
        self
    }

    pub fn ok<T: Into<Vec<u8>>>(self, stdout: T) {
        self.assert().success().stdout(stdout.into()).stderr("");
    }

    pub fn err<T: Into<Vec<u8>>>(self, stderr: T) {
        self.assert().failure().stderr(stderr.into());
    }

    fn assert(self) -> Assert {
        Command::new(self.bin)
            .args(self.args)
            .timeout(Duration::from_millis(500))
            .write_stdin(self.stdin.unwrap_or_default())
            .assert()
    }
}
