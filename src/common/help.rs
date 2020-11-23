use crate::color::spec_color;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

const DOUBLE_LINE_PREFIX: &str = "====";
const SIMPLE_LINE_PREFIX: &str = "----";
const PADDED_BLOCK_PREFIX: &str = "    ";
const SHELL_PREFIX: &str = "$>";

const CODE_CHAR: char = '`';
const COMMENT_CHAR: char = '#';

const PRIMARY_COLOR: Color = Color::Yellow;
const SECONDARY_COLOR: Color = Color::Cyan;
const CODE_COLOR: Color = Color::Green;

pub fn highlight_help<O: Write + WriteColor>(output: &mut O, text: &str) -> Result<()> {
    let mut in_heading = false;
    let mut in_padded_block_after_line = false;

    for line in text.lines() {
        if line.starts_with(DOUBLE_LINE_PREFIX) {
            output.set_color(&spec_color(PRIMARY_COLOR))?;
            in_heading = !in_heading;
            write!(output, "{}", line)?;
        } else if in_heading {
            output.set_color(&spec_color(PRIMARY_COLOR))?;
            write!(output, "{}", line)?;
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
    use super::*;
    use crate::testing::{ColoredOuput, OutputChunk};
    use indoc::indoc;

    #[test]
    fn highlights_help() {
        let help = indoc! {"
            =========
             Heading
            =========

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
                
            =========
             Heading
            =========
                
            Text.
        "};

        let mut ouput = ColoredOuput::new();
        highlight_help(&mut ouput, help).unwrap();
        assert_eq!(
            ouput.chunks(),
            &[
                OutputChunk::color(Color::Yellow, "========="),
                OutputChunk::plain("\n"),
                OutputChunk::color(Color::Yellow, " Heading"),
                OutputChunk::plain("\n"),
                OutputChunk::color(Color::Yellow, "========="),
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
                OutputChunk::plain("      2\n    \nText.\n\n    "),
                OutputChunk::color(Color::Cyan, "$>"),
                OutputChunk::color(Color::Green, " ls -la"),
                OutputChunk::plain("\n    "),
                OutputChunk::color(Color::Cyan, "$>"),
                OutputChunk::color(Color::Green, " ls -la "),
                OutputChunk::plain("# Shell comment\n    \n"),
                OutputChunk::color(Color::Yellow, "========="),
                OutputChunk::plain("\n"),
                OutputChunk::color(Color::Yellow, " Heading"),
                OutputChunk::plain("\n"),
                OutputChunk::color(Color::Yellow, "========="),
                OutputChunk::plain("\n    \nText.\n")
            ]
        );
    }
}
