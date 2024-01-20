use crate::args::GlobalArgs;
use crate::command::Meta;
use crate::command_meta;
use crate::io::Writer;
use anyhow::anyhow;
use anyhow::Result;

pub const META: Meta = command_meta! {
    name: "seq",
    args: Args,
    run: run,
};

/// Print sequence of numbers as lines
#[derive(clap::Args)]
struct Args {
    /// First number in sequence.
    #[arg(default_value_t = 1, allow_negative_numbers = true)]
    first: i128,

    /// Last number in sequence.
    ///
    /// If not specified, the sequence is unbounded.
    #[arg(allow_negative_numbers = true)]
    last: Option<i128>,

    /// Increment between numbers in sequence.
    ///
    /// Defaults to 1 for increasing sequence, -1 for decreasing one.
    #[arg(short, long, allow_negative_numbers = true)]
    step: Option<i128>,
}

#[allow(clippy::too_many_lines)]
fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let first = args.first;
    let step = args.step;

    let mut value = first;
    let mut writer = Writer::from(global_args);
    let mut formatter = Formatter::new();

    match args.last {
        Some(last) if first < last => {
            let step = step.unwrap_or(1);
            while value <= last {
                writer.write_line(formatter.format(value))?;
                value += step;
            }
        }
        Some(last) if first > last => {
            let step = step.unwrap_or(-1);
            while value >= last {
                writer.write_line(formatter.format(value))?;
                value += step;
            }
        }
        Some(_) => {
            // first == last
            writer.write_line(formatter.format(value))?;
        }
        None => {
            let step = step.unwrap_or(1);
            loop {
                writer.write_line(formatter.format(value))?;
                if let Some(new_value) = value.checked_add(step) {
                    value = new_value;
                } else {
                    return Err(anyhow!("number sequence overflown interger limit"));
                }
            }
        }
    }

    Ok(())
}

struct Formatter {
    buffer: Vec<u8>,
}

impl Formatter {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(64), // Enough for i128 digits
        }
    }

    fn format(&mut self, value: i128) -> &[u8] {
        self.buffer.clear();
        itoap::write_to_vec(&mut self.buffer, value);
        &self.buffer
    }
}
