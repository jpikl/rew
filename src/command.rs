use anyhow::Result;
use clap::ArgMatches;
use clap::Command;

pub struct CommandMeta {
    pub name: &'static str,
    pub build: fn() -> Command,
    pub run: fn(&ArgMatches) -> Result<()>,
}

#[macro_export]
macro_rules! command_meta {
    (name: $name:literal, args: $args:ident, run: $run:ident,) => {
        CommandMeta {
            name: $name,
            build: || -> clap::Command {
                use clap::Args as ClapArgs;
                $args::augment_args(clap::Command::new($name))
            },
            run: |matches| -> anyhow::Result<()> {
                use clap::FromArgMatches;
                let global_args = $crate::args::GlobalArgs::from_arg_matches(matches)?;
                let args = $args::from_arg_matches(matches)?;
                $run(global_args, args)
            },
        }
    };
}
