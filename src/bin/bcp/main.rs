use crate::cli::Cli;
use common::{detect_color, write_error};
use std::io::{self, Stdin};
use std::process;
use structopt::StructOpt;
use termcolor::StandardStream;

mod cli;

const ERR_IO: i32 = 2;

fn main() {
    let cli: Cli = Cli::from_args(); // Explicit variable type, because IDE is unable to detect it.
    let color = detect_color(cli.color);

    let mut stdin = io::stdin();
    let mut stdout = StandardStream::stdout(color);
    let mut stderr = StandardStream::stderr(color);

    if let Some(io_error) = run(&cli, &mut stdin, &mut stdout, &mut stderr).err() {
        write_error(&mut stderr.lock(), &io_error).expect("Failed to write to stderr!");
        process::exit(ERR_IO);
    }
}

fn run(
    cli: &Cli,
    stdin: &mut Stdin,
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
) -> Result<(), io::Error> {
    unimplemented!()
}
