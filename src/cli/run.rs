use super::CommandItem;
use super::Group;
use super::OptArg;
use super::PosArg;
use super::args::Args;
use super::builders::FlagBuilder;
use super::types::Command;
use super::types::Flag;

pub const HELP: Flag = FlagBuilder::new("help")
    .short('h')
    .long("help")
    .description("Print short help (-h) or detailed help (--help)")
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

        for (group, options) in self.command.grouped_options() {
            print_item_group(group, &options);
        }

        for (group, positionals) in self.command.grouped_positionals() {
            print_item_group(group, &positionals);
        }

        for (group, commands) in self.command.grouped_commands() {
            print_item_group(group, &commands);
        }
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
    fn grouped_options(&self) -> Vec<(&Group, Vec<&OptArg>)> {
        group_items(self.options)
    }

    fn grouped_positionals(&self) -> Vec<(&Group, Vec<&PosArg>)> {
        group_items(self.positionals)
    }

    fn grouped_commands(&self) -> Vec<(&Group, Vec<&Command>)> {
        group_items(self.commands)
    }
}

fn print_item_group<T: CommandItem>(group: &Group, items: &[&T]) {
    println!();
    println!("{}:", group.name);

    let width = items
        .iter()
        .map(|item| item.usage().len())
        .max()
        .unwrap_or_default();

    for item in items {
        let usage = item.usage();
        let padding = " ".repeat(width - usage.len());
        println!("  {usage}{padding}  {}", item.description());
    }
}

fn group_items<T: CommandItem>(items: &[T]) -> Vec<(&Group, Vec<&T>)> {
    let mut grouped: Vec<(&Group, Vec<&T>)> = Vec::new();

    for item in items.iter() {
        let group = grouped.iter_mut().find(|(g, _)| *g == item.group());

        match group {
            Some((_, items)) => items.push(item),
            None => grouped.push((item.group(), vec![item])),
        }
    }

    grouped
}
