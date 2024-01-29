use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use anyhow::Result;
use bstr::ByteSlice;
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

    let trim_separator = context.separator().trim_fn();
    let output_separator = context.separator().as_byte();
    let input_separator = args.separator;

    let mut last_block_terminated = false;
    let mut start_next_block_terminated = false;

    while let Some(block) = reader.read_block()? {
        if start_next_block_terminated {
            writer.write_block(&[output_separator])?;
        }

        start_next_block_terminated = block.last_byte() == Some(output_separator);
        last_block_terminated = trim_separator(block).last_byte() == Some(input_separator);
        let mut remainder = &mut block[..];

        while let Some(pos) = memchr(input_separator, remainder) {
            remainder[pos] = output_separator;
            remainder = &mut remainder[(pos + 1)..];
        }

        if start_next_block_terminated {
            writer.write_block(trim_separator(block))?;
        } else {
            writer.write_block(block)?;
        }
    }

    if !last_block_terminated || !args.ignore_trailing {
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
    use claims::assert_err_eq;
    use claims::assert_ok_eq;

    #[test]
    fn parse_separator() {
        assert_ok_eq!(super::parse_separator("a"), b'a');
        assert_err_eq!(
            super::parse_separator(""),
            "value must be a single character"
        );
        assert_err_eq!(
            super::parse_separator("ab"),
            "value must be a single character"
        );
        assert_err_eq!(
            super::parse_separator("á"),
            "multi-byte characters are not supported"
        );
    }
}
