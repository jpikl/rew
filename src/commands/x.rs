use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::pattern::Pattern;
use crate::pattern::SimpleItem;
use anyhow::Result;

pub const META: Meta = command_meta! {
    name: "x",
    group: Group::Transformers,
    args: Args,
    run: run,
};

/// Compose parallel shell pipelines using a pattern.
#[derive(clap::Args)]
struct Args {
    /// Composition pattern.
    ///
    /// `abc`             Constant  
    /// `{}`              Empty expression     
    /// `{cmd}`           Expression with a filter command
    /// `{cmd a b}`       Expression with a filter command and args
    /// `{x|y a b|z}`     Expression with a command pipeline
    /// `a{}b{x|y a b}c`  Mixed constant and expresions.
    #[arg(verbatim_doc_comment)]
    pattern: String,

    /// Escape character for the pattern.
    #[arg(short, long, value_name = "CHAR", default_value_t = '\\')]
    escape: char,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let pattern = Pattern::parse(&args.pattern, args.escape)?;

    if let Some(pattern) = pattern.try_simplify() {
        let mut reader = context.line_reader();
        let mut writer = context.writer();

        while let Some(line) = reader.read_line()? {
            for item in pattern.items() {
                match item {
                    SimpleItem::Constant(value) => writer.write(value.as_bytes())?,
                    SimpleItem::Expression => writer.write(line)?,
                }
            }
            writer.write_separator()?;
        }

        return Ok(());
    }

    unimplemented!();
}
