use super::args::Args;
use super::builders::FlagBuilder;
use super::types::Command;
use super::types::Flag;

pub const HELP: Flag = FlagBuilder::new()
    .short('h')
    .long("help")
    .description("Print help (see more with '--help')")
    .done();

pub const VERSION: Flag = FlagBuilder::new()
    .short('V')
    .long("version")
    .description("Print version")
    .done();

impl Command {
    pub fn run(&self, args: Args) -> anyhow::Result<()> {
        if args.get(&HELP) {
            println!("{}", self.description);
            return Ok(());
        }

        if args.get(&VERSION) {
            println!("{}", self.version);
            return Ok(());
        }

        (self.run)(args)
    }
}
