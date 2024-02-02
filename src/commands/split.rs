use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use memchr::memchr;

pub const META: Meta = command_meta! {
    name: "split",
    group: Group::Transformers,
    args: Args,
    run: run,
};

/// Split input into lines using a separator.
#[derive(clap::Args)]
struct Args {
    //// Separator (single byte character).
    #[arg(value_parser = parse_separator)]
    separator: u8,

    /// Ignore trailing separator at the end of input.
    #[arg(short = 't', long)]
    ignore_trailing: bool,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let mut reader = context.block_reader();
    let mut writer = context.writer();

    let input_separator = args.separator;
    let output_separator = context.separator().as_byte();
    let trim_output_separator = context.separator().trim_fn();

    let mut ending_separator_written = false;
    let mut start_next_block_separated = false;

    while let Some(mut block) = reader.read_block()? {
        if start_next_block_separated {
            writer.write_block(&[output_separator])?;
            ending_separator_written = true;
        }

        let trimmed_block_len = trim_output_separator(block).len();
        if trimmed_block_len < block.len() {
            // Write the trimmed separator once we know there is more data
            block = &mut block[..trimmed_block_len];
            start_next_block_separated = true;
        } else {
            start_next_block_separated = false;
        }

        if trimmed_block_len > 0 {
            let mut remainder = &mut block[..];

            while let Some(pos) = memchr(input_separator, remainder) {
                remainder[pos] = output_separator;
                remainder = &mut remainder[(pos + 1)..];
            }

            writer.write_block(block)?;
            ending_separator_written = block[trimmed_block_len - 1] == output_separator;
        }
    }

    if !ending_separator_written || !args.ignore_trailing {
        writer.write_block(&[output_separator])?;
    }

    Ok(())
}

fn parse_separator(string: &str) -> std::result::Result<u8, &'static str> {
    if string.chars().count() != 1 {
        Err("value must be a single character")
    } else if string.len() != 1 {
        Err("multi-byte characters are not supported")
    } else {
        Ok(string.as_bytes()[0])
    }
}

#[cfg(test)]
mod tests {
    use super::parse_separator as parse;
    use claims::assert_err_eq;
    use claims::assert_ok_eq;

    #[test]
    fn parse_separator() {
        assert_ok_eq!(parse("a"), b'a');
        assert_err_eq!(parse(""), "value must be a single character");
        assert_err_eq!(parse("ab"), "value must be a single character");
        assert_err_eq!(parse("á"), "multi-byte characters are not supported");
    }
}