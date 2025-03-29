use io::LineReader;
use std::error::Error;

mod io;

fn main() -> Result<(), Box<dyn Error>> {
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
