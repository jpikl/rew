use crate::color::{detect_color, HasColor};
use crate::output::write_error;
use std::io::{self, Stdin};
use std::process;
use termcolor::StandardStream;

pub const ERR_IO: i32 = 2;

pub type Result = std::io::Result<()>;

pub fn exec_run<R, C>(run: R, cli: C)
where
    R: FnOnce(C, &mut Stdin, &mut StandardStream, &mut StandardStream) -> Result,
    C: HasColor,
{
    let color = detect_color(cli.color());

    let mut stdin = io::stdin();
    let mut stdout = StandardStream::stdout(color);
    let mut stderr = StandardStream::stderr(color);

    if let Some(io_error) = run(cli, &mut stdin, &mut stdout, &mut stderr).err() {
        write_error(&mut stderr.lock(), &io_error).expect("Failed to write to stderr!");
        process::exit(ERR_IO);
    }
}
