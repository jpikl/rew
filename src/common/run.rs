use std::io::{Stdin, StdinLock};
use std::{io, process};

use clap::Parser;
use termcolor::{ColorChoice, StandardStream, StandardStreamLock};

use crate::color::choose_color;
use crate::output::write_error;

pub const EXIT_CODE_OK: i32 = 0;
pub const EXIT_CODE_IO_ERROR: i32 = 1;
pub const EXIT_CODE_CLI_ERROR: i32 = 2;

pub type Result = io::Result<i32>;

pub trait Options: Parser {
    fn color(&self) -> Option<ColorChoice>;
}

pub struct Io {
    stdin: Stdin,
    stdout: StandardStream,
    stderr: StandardStream,
}

impl Io {
    pub fn new(color: ColorChoice) -> Self {
        Self {
            stdin: io::stdin(),
            stdout: StandardStream::stdout(color),
            stderr: StandardStream::stderr(color),
        }
    }

    pub fn stdin(&self) -> StdinLock {
        self.stdin.lock()
    }

    pub fn stdout(&self) -> StandardStreamLock {
        self.stdout.lock()
    }

    pub fn stderr(&self) -> StandardStreamLock {
        self.stderr.lock()
    }
}

pub fn exec_run<O, R>(run: R)
where
    O: Options,
    R: FnOnce(&O, &Io) -> Result,
{
    let options = O::parse();
    let color = choose_color(options.color());
    let io = Io::new(color);

    let exit_code = match run(&options, &io) {
        Ok(exit_code) => exit_code,
        Err(io_error) => {
            write_error(&mut io.stderr(), &io_error).expect("Failed to write to stderr!");
            EXIT_CODE_IO_ERROR
        }
    };

    process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use claim::*;

    use super::*;

    #[test]
    fn io() {
        let io = Io::new(ColorChoice::Never);
        assert_ok!(io.stdin().read_exact(&mut []));
        assert_ok!(io.stdout().flush());
        assert_ok!(io.stderr().flush());
    }
}
