use std::io::{self, Stdin, StdinLock};
use termcolor::{ColorChoice, StandardStream, StandardStreamLock};

use crate::io::Io;

pub struct SystemIo {
    stdin: Stdin,
    stdout: StandardStream,
    stderr: StandardStream,
}

impl SystemIo {
    pub fn new(color: ColorChoice) -> Self {
        Self {
            stdin: io::stdin(),
            stdout: StandardStream::stdout(color),
            stderr: StandardStream::stderr(color),
        }
    }
}

impl<'a> Io<'a> for SystemIo {
    type StdinLock = StdinLock<'a>;
    type StdoutLock = StandardStreamLock<'a>;
    type StderrLock = StandardStreamLock<'a>;

    fn stdin(&'a self) -> Self::StdinLock {
        self.stdin.lock()
    }

    fn stdout(&'a self) -> Self::StdoutLock {
        self.stdout.lock()
    }

    fn stderr(&'a self) -> Self::StderrLock {
        self.stderr.lock()
    }
}
