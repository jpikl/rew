mod cli;
mod io;

use cli::Args;
use cli::Command;
use cli::CommandBuilder;
use cli::HELP;
use cli::VERSION;
use io::LineReader;

const REW: Command = CommandBuilder::new()
    .name("linecount")
    .description("Count lines and bytes in a file")
    .version("1.0.0")
    .options(&[HELP.arg, VERSION.arg])
    .run(run)
    .done();

fn run(_: Args) -> anyhow::Result<()> {
    let mut reader = LineReader::new(std::io::stdin(), io::Separator::Newline, 32 * 1024);
    let mut lines = 0;
    let mut bytes = 0;

    while let Some(line) = reader.read_line()? {
        lines += 1;
        bytes += line.len();
    }

    println!("lines: {lines}");
    println!("bytes: {bytes}");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    REW.run(REW.parse_args()?)
}
