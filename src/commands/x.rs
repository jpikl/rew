use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::pattern::Pattern;
use anyhow::Result;

pub const META: Meta = command_meta! {
    name: "x",
    group: Group::Transformers,
    args: Args,
    run: run,
};

/// Compose parallel shell pipelines using a pattern
#[derive(clap::Args)]
struct Args {
    /// Composition pattern
    pattern: String,

    /// Escape character for the pattern
    #[arg(short, long, value_name = "CHAR", default_value_t = '\\')]
    escape: char,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let pattern = Pattern::parse(&args.pattern, args.escape)?;
    context.writer().write_line(pattern.to_string().as_bytes())
}
