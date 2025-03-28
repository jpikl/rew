mod io;

use io::LineReader;
use io::Separator;

fn main() -> anyhow::Result<()> {
    println!("method: rust");
    let mut reader = LineReader::new(std::io::stdin(), Separator::Newline, 32 * 1024);
    let mut lines = 0;
    let mut bytes = 0;
    while let Some(line) = reader.read_line()? {
        bytes += line.len();
        lines += 1;
    }
    println!("lines: {lines}");
    println!("bytes: {bytes}");
    Ok(())
}
