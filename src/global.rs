use crate::cli::EnumItem;
use crate::cli::Flag;
use crate::cli::FlagBuilder;
use crate::cli::Group;
use crate::cli::Opt;
use crate::cli::OptBuilder;
use crate::cli::ParseArg;
use crate::cli::ParseArgResult;
use crate::impl_enum;
use std::borrow::Cow;
use std::io::IsTerminal;

const GLOBAL_OPTIONS: Group = Group {
    name: "Global options",
    description: "Options shared by all commands.",
};

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library https://github.com/Freaky/rust-linereader.
const DEFAULT_BUF_SIZE: &str = "32K";

pub const NULL: Flag = FlagBuilder::new("null")
    .short('0')
    .long("null")
    .description("Line delimiter is NUL, not newline.")
    .environment("REW_NULL")
    .group(&GLOBAL_OPTIONS)
    .done();

pub const BUF_SIZE: Opt<ByteSize> = OptBuilder::new("buf-size")
    .long("buf-size")
    .description("Size of a buffer used for IO operations.")
    .description_ex(&[
        "Supports K/M/G multipliers (1K = 1024, 1M = 1024K, 1G = 1024M).",
        "Smaller values will reduce memory consumption but could negatively affect throughput.",
        "Larger values will increase memory consumption but may improve throughput in some cases.",
        "Some commands will abort execution if there is an input line above this limit.",
    ])
    .environment("REW_BUF_SIZE")
    .value_name("SIZE")
    .default(DEFAULT_BUF_SIZE)
    .group(&GLOBAL_OPTIONS)
    .done();

pub const BUF_MODE: Opt<BufMode> = OptBuilder::new_enum("buf-mode")
    .long("buf-mode")
    .description("Output buffering mode.")
    .environment("REW_BUF_MODE")
    .value_name("MODE")
    .group(&GLOBAL_OPTIONS)
    .done();

#[derive(Default, Clone)]
pub struct ByteSize(pub usize);

impl ParseArg for ByteSize {
    fn parse_arg(raw_value: Cow<str>) -> ParseArgResult {
        match parse_byte_size(raw_value.as_ref()) {
            Ok(size) => Ok(Box::new(ByteSize(size))),
            Err(err) => Err(err),
        }
    }
}

fn parse_byte_size(value: &str) -> Result<usize, String> {
    let (value, multiplier) = if let Some(value) = value.strip_suffix("G") {
        (value, 1024 * 1024 * 1024)
    } else if let Some(value) = value.strip_suffix("M") {
        (value, 1024 * 1024)
    } else if let Some(value) = value.strip_suffix("K") {
        (value, 1024)
    } else {
        (value, 1)
    };
    match value.trim_end().parse::<usize>() {
        Ok(value) => Ok(value * multiplier),
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Clone, Copy)]
pub enum BufMode {
    Line,
    Full,
}

impl Default for BufMode {
    fn default() -> Self {
        if std::io::stdout().is_terminal() {
            Self::Line
        } else {
            Self::Full
        }
    }
}

impl_enum! (BufMode, {
    Line: {
        name: "line",
        description: [
            "Writes to stdout after a line is processed or when the output buffer is full.",
            "Enabled by default when stdout is TTY (for interactive usage).",
        ],
    },
    Full: {
        name: "full",
        description: [
            "Writes to stdout only when the output buffer is full.",
            "Enabled by default when stdout is not TTY (for maximal throughput).",
        ],
    },
});
