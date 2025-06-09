use crate::cli::ArgUsage;
use crate::cli::Command;
use crate::cli::Context;
use crate::cli::ErrorKind;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use std::io::Write;

pub const VERSION: Flag = FlagBuilder::new()
    .short('V')
    .long("version")
    .description("Print version.")
    .run(print_version)
    .done();

fn print_version<'a>(context: &Context<'a>, _usage: &ArgUsage) -> Result<(), ErrorKind<'a>> {
    let command = context.commands.current();
    command.print_version(&mut std::io::stdout().lock())?;
    Ok(())
}

impl Command<'_> {
    pub fn print_version(&self, writer: &mut impl Write) -> std::io::Result<()> {
        write!(writer, "{}", self.name)?;

        if let Some(version) = self.version {
            write!(writer, " {version}")?;
        }

        writeln!(writer)
    }
}
