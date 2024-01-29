use crate::args::BufMode;
use crate::args::GlobalArgs;
use crate::io::BlockReader;
use crate::io::LineReader;
use crate::io::Separator;
use crate::io::Writer;
use anyhow::Result;
use clap::ArgMatches;
use clap::Command;
use derive_more::Display;
use std::io::stdin;
use std::io::stdout;
use std::io::StdinLock;
use std::io::StdoutLock;

#[derive(Display)]
pub enum Group {
    #[display("Mapper commands")]
    Mappers,
    #[display("Filter commands")]
    Filters,
    #[display("Transform commands")]
    Transformers,
    #[display("Generator commands")]
    Generators,
}

pub struct Meta {
    pub name: &'static str,
    pub group: Group,
    pub build: fn() -> Command,
    pub run: fn(&ArgMatches) -> Result<()>,
}

#[macro_export]
macro_rules! command_meta {
    (name: $name:literal, group: $group:expr, args: $args:ident, run: $run:ident,) => {
        $crate::command::Meta {
            name: $name,
            group: $group,
            build: || -> clap::Command {
                use clap::Args as ClapArgs;
                $args::augment_args(clap::Command::new($name))
            },
            run: |matches| -> anyhow::Result<()> {
                use clap::FromArgMatches;
                let global_args = $crate::args::GlobalArgs::from_arg_matches(matches)?;
                let context = $crate::command::Context::from(global_args);
                let args = $args::from_arg_matches(matches)?;
                $run(&context, &args)
            },
        }
    };
}

pub struct Context(GlobalArgs);

impl From<GlobalArgs> for Context {
    fn from(args: GlobalArgs) -> Self {
        Self(args)
    }
}

impl Context {
    #[allow(clippy::unused_self)]
    pub fn raw_reader(&self) -> StdinLock<'_> {
        stdin().lock()
    }

    pub fn block_reader(&self) -> BlockReader<StdinLock<'_>> {
        BlockReader::new(self.raw_reader(), self.buf_size())
    }

    pub fn line_reader(&self) -> LineReader<StdinLock<'_>> {
        LineReader::new(self.raw_reader(), self.separator(), self.buf_size())
    }

    #[allow(clippy::unused_self)]
    pub fn raw_writer(&self) -> StdoutLock<'_> {
        stdout().lock()
    }

    pub fn writer(&self) -> Writer<StdoutLock<'_>> {
        Writer::new(
            self.raw_writer(),
            self.separator(),
            self.0.buf_mode == BufMode::Full,
            self.buf_size(),
        )
    }

    pub fn zeroed_buf(&self) -> Vec<u8> {
        vec![0u8; self.buf_size()]
    }

    pub fn uninit_buf(&self) -> Vec<u8> {
        Vec::with_capacity(self.buf_size())
    }

    pub fn buf_size(&self) -> usize {
        self.0.buf_size
    }

    pub fn separator(&self) -> Separator {
        if self.0.null {
            Separator::Null
        } else {
            Separator::Newline
        }
    }
}
