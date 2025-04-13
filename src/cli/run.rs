use super::args::Args;
use super::builders::FlagBuilder;
use super::types::Command;
use super::types::Flag;

pub const HELP: Flag = FlagBuilder::new("help")
    .short('h')
    .long("help")
    .description("Print help (see more with '--help')")
    .done();

pub const VERSION: Flag = FlagBuilder::new("version")
    .short('V')
    .long("version")
    .description("Print version")
    .done();

pub struct Runner<'a> {
    pub binary: String,
    pub command: &'a Command,
    pub parents: Vec<&'a Command>,
    pub args: Args,
}

impl Runner<'_> {
    pub fn run(self) -> anyhow::Result<()> {
        if self.args.get(&HELP) {
            self.print_help();
            return Ok(());
        }

        if self.args.get(&VERSION) {
            self.print_version();
            return Ok(());
        }

        (self.command.run)(self.args)
    }

    fn print_help(&self) {
        println!("{}", self.command.description);

        for description in self.command.description_ex {
            println!("{description}");
        }

        println!();
        println!("Usage: {}{}", self.binary, self.command.usage_params());
    }

    fn print_version(&self) {
        for command in &self.parents {
            print!("{} ", command.name);
        }

        print!("{}", self.command.name);

        if let Some(version) = self.command.version {
            print!(" {version}");
        }

        println!();
    }
}

impl Command {
    fn usage_params(&self) -> String {
        let mut usage = String::new();

        if !self.options.is_empty() {
            usage.push_str(" [OPTIONS]");
        }

        if !self.positionals.is_empty() {
            usage.push_str(" [ARGUMENTS]");
        }

        if !self.commands.is_empty() {
            usage.push_str(" <COMMAND>");
        }

        usage
    }
}
