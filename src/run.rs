use crate::color::detect_color;
use crate::io::mem::{MemoryIo, OutputChunk};
use crate::io::sys::SystemIo;
use crate::io::Io;
use crate::output::write_error;
use std::{io, process, result};
use structopt::{clap, StructOpt};
use termcolor::ColorChoice;

pub const EXIT_CODE_OK: i32 = 0;
pub const EXIT_CODE_CLI_ERROR: i32 = 1;
pub const EXIT_CODE_IO_ERROR: i32 = 2;

pub type Result = io::Result<i32>;

pub trait Cli: StructOpt + Sized {
    fn from_custom_args_safe(args: &[&str]) -> result::Result<Self, clap::Error> {
        Self::from_iter_safe([&["cmd"][..], args].concat())
    }

    fn color(&self) -> Option<ColorChoice>;
}

pub struct Runner<C: Cli> {
    cli: C,
    io: SystemIo,
}

impl<C: Cli> Runner<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exec<'a, R>(&'a self, run: R)
    where
        R: FnOnce(&'a C, &'a SystemIo) -> Result,
    {
        let exit_code = match run(&self.cli, &self.io) {
            Ok(exit_code) => exit_code,
            Err(io_error) => {
                write_error(&mut self.io.stderr(), &io_error).expect("Failed to write to stderr!");
                EXIT_CODE_IO_ERROR
            }
        };
        process::exit(exit_code);
    }
}

impl<C: Cli> Default for Runner<C> {
    fn default() -> Self {
        let cli = C::from_args();
        let color = detect_color(cli.color());
        let io = SystemIo::new(color);
        Self { cli, io }
    }
}

pub struct TestRunner<C: Cli> {
    cli: C,
    io: MemoryIo,
}

impl<C: Cli> TestRunner<C> {
    pub fn new(args: &[&str], input: &'static [u8]) -> result::Result<Self, clap::Error> {
        let cli = Cli::from_custom_args_safe(args)?;
        let io = MemoryIo::new(input);
        Ok(Self { cli, io })
    }

    pub fn stdout(&self) -> Vec<OutputChunk> {
        self.io.stdout_chunks()
    }

    pub fn stderr(&self) -> Vec<OutputChunk> {
        self.io.stderr_chunks()
    }

    pub fn exec<'a, R>(&'a self, run: R) -> i32
    where
        R: FnOnce(&'a C, &'a MemoryIo) -> Result,
    {
        match run(&self.cli, &self.io) {
            Ok(exit_code) => exit_code,
            Err(_) => EXIT_CODE_IO_ERROR,
        }
    }
}
