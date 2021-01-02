use crate::color::spec_color;
use crate::utils::str_from_utf8;
use lazy_static::lazy_static;
use std::io::{Result, Write};
use std::sync::Mutex;
use termcolor::{Buffer, Color, WriteColor};

const DOUBLE_LINE_PREFIX: &str = "===";
const SIMPLE_LINE_PREFIX: &str = "---";
const PADDED_BLOCK_PREFIX: &str = "    ";
const SHELL_PREFIX: &str = "$>";

const CODE_CHAR: char = '`';
const COMMENT_CHAR: char = '#';

const PRIMARY_COLOR: Color = Color::Yellow;
const SECONDARY_COLOR: Color = Color::Cyan;
const CODE_COLOR: Color = Color::Green;

static mut STATIC_HIGHLIGHTS: Vec<String> = Vec::new();

lazy_static! {
    static ref STATIC_HIGHLIGHTS_MUTEX: Mutex<()> = Mutex::new(());
    static ref STATIC_HIGHLIGHTS_ENABLED: bool = atty::is(atty::Stream::Stdout)
        && std::env::args().any(|arg| arg == "-h" || arg == "--help");
}

pub fn highlight_static(text: &str) -> &str {
    if *STATIC_HIGHLIGHTS_ENABLED {
        match highlight_to_string(text) {
            Ok(result) => unsafe {
                // Well, this is ugly but the current usage should be actually safe:
                // 1) It's used only by cli.rs to generate static strings for clap attributes.
                // 2) Values are never modified after being pushed to vector.
                let _lock = STATIC_HIGHLIGHTS_MUTEX.lock(); // Lock might not be actually needed in our case
                STATIC_HIGHLIGHTS.push(result);
                STATIC_HIGHLIGHTS
                    .last()
                    .expect("Expected at least one static highlight result")
                    .as_str()
            },
            Err(_) => text,
        }
    } else {
        text
    }
}

fn highlight_to_string(text: &str) -> Result<String> {
    let mut buffer = Buffer::ansi();
    highlight(&mut buffer, text)?;
    str_from_utf8(buffer.as_slice()).map(String::from)
}

pub fn highlight<O: Write + WriteColor>(output: &mut O, text: &str) -> Result<()> {
    let mut in_heading = false;
    let mut in_padded_block_after_line = false;

    for line in text.lines() {
        if line.starts_with(DOUBLE_LINE_PREFIX) {
            output.set_color(&spec_color(PRIMARY_COLOR))?;
            in_heading = !in_heading;
            write!(output, "{}", line)?;
        } else if in_heading {
            if line.is_empty() {
                in_heading = false;
            } else {
                output.set_color(&spec_color(PRIMARY_COLOR))?;
                write!(output, "{}", line)?;
            }
        } else if let Some(block) = line.strip_prefix(PADDED_BLOCK_PREFIX) {
            write!(output, "{}", PADDED_BLOCK_PREFIX)?;

            if let Some(command) = block.strip_prefix(SHELL_PREFIX) {
                output.set_color(&spec_color(SECONDARY_COLOR))?;
                write!(output, "{}", SHELL_PREFIX)?;
                output.set_color(&spec_color(CODE_COLOR))?;

                if let Some(comment_index) = command.rfind(COMMENT_CHAR) {
                    write!(output, "{}", &command[..comment_index])?;
                    output.reset()?;
                    write!(output, "{}", &command[comment_index..])?;
                } else {
                    write!(output, "{}", command)?;
                }
            } else if block.starts_with(SIMPLE_LINE_PREFIX) {
                in_padded_block_after_line = true;
                output.set_color(&spec_color(SECONDARY_COLOR))?;
                write!(output, "{}", block)?;
            } else if in_padded_block_after_line {
                highlight_code(output, block)?;
            } else {
                output.set_color(&spec_color(SECONDARY_COLOR))?;
                write!(output, "{}", block)?;
            }
        } else {
            in_padded_block_after_line = false;
            highlight_code(output, line)?;
        }

        output.reset()?;
        writeln!(output)?;
    }

    Ok(())
}

fn highlight_code<O: Write + WriteColor>(output: &mut O, line: &str) -> Result<()> {
    let mut in_code = false;
    let mut last_index = 0;

    for (index, char) in line.char_indices() {
        if char == CODE_CHAR {
            write!(output, "{}", &line[last_index..index])?;
            if in_code {
                output.reset()?;
            } else {
                output.set_color(&spec_color(CODE_COLOR))?;
            }
            last_index = index + 1;
            in_code = !in_code;
        }
    }

    write!(output, "{}", &line[last_index..])
}

#[cfg(test)]
mod tests {
    use crate::testing::{ColoredOuput, OutputChunk};
    use claim::*;
    use indoc::indoc;

    const SAMPLE_HELP: &str = indoc! {"
        =========
         Heading
        =========

        Text.

        === Heading ===

        Text.
        Text with `code`.
        Text with `code` and `more code`.

            KEY    VALUE
            ------------
            `a`      1
            `b`      2

        Text.

            $> ls -la
            $> ls -la # Shell comment
    "};

    #[test]
    fn highlight_to_string() {
        use super::*;
        assert_gt!(highlight_to_string(SAMPLE_HELP).unwrap().len(), 0);
    }

    #[test]
    fn highlight() {
        use super::*;

        let mut ouput = ColoredOuput::new();
        highlight(&mut ouput, SAMPLE_HELP).unwrap();
        assert_eq!(
            ouput.chunks(),
            &[
                OutputChunk::color(Color::Yellow, "========="),
                OutputChunk::plain("\n"),
                OutputChunk::color(Color::Yellow, " Heading"),
                OutputChunk::plain("\n"),
                OutputChunk::color(Color::Yellow, "========="),
                OutputChunk::plain("\n\nText.\n\n"),
                OutputChunk::color(Color::Yellow, "=== Heading ==="),
                OutputChunk::plain("\n\nText.\nText with "),
                OutputChunk::color(Color::Green, "code"),
                OutputChunk::plain(".\nText with "),
                OutputChunk::color(Color::Green, "code"),
                OutputChunk::plain(" and "),
                OutputChunk::color(Color::Green, "more code"),
                OutputChunk::plain(".\n\n    "),
                OutputChunk::color(Color::Cyan, "KEY    VALUE"),
                OutputChunk::plain("\n    "),
                OutputChunk::color(Color::Cyan, "------------"),
                OutputChunk::plain("\n    "),
                OutputChunk::color(Color::Green, "a"),
                OutputChunk::plain("      1\n    "),
                OutputChunk::color(Color::Green, "b"),
                OutputChunk::plain("      2\n\nText.\n\n    "),
                OutputChunk::color(Color::Cyan, "$>"),
                OutputChunk::color(Color::Green, " ls -la"),
                OutputChunk::plain("\n    "),
                OutputChunk::color(Color::Cyan, "$>"),
                OutputChunk::color(Color::Green, " ls -la "),
                OutputChunk::plain("# Shell comment\n"),
            ]
        );
    }
}
