use clap::Args;
use clap::ValueEnum;
use derive_more::Display;
use derive_more::IsVariant;
use std::io::stdout;
use std::io::IsTerminal;

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library https://github.com/Freaky/rust-linereader.
const DEFAULT_BUF_SIZE: usize = 32 * 1024;

pub const ENV_NULL: &str = "REW_NULL";
pub const ENV_BUF_MODE: &str = "REW_BUF_MODE";
pub const ENV_BUF_SIZE: &str = "REW_BUF_SIZE";

#[derive(Clone, Copy, ValueEnum, Display, Debug, IsVariant, PartialEq, Eq)]
pub enum BufMode {
    #[display("line")]
    Line,
    #[display("full")]
    Full,
}

impl Default for BufMode {
    fn default() -> Self {
        if stdout().is_terminal() {
            Self::Line
        } else {
            Self::Full
        }
    }
}

#[derive(Args, Default, Debug, Clone, Eq, PartialEq)]
pub struct GlobalArgs {
    /// Line delimiter is NUL, not newline.
    #[arg(global = true, short = '0', long, env = ENV_NULL)]
    pub null: bool,

    /// Output buffering mode.
    ///
    /// - `line` - Writes to stdout after a line was processed or when the output buffer is full.
    /// - `full` - Writes to stdout only when the output buffer is full.
    ///
    /// Defaults to `line` when stdout is TTY (for interactive usage), otherwise is `full` (for maximal throughput).
    ///
    /// Size of the output buffer can be configured through the `--buf-size` global option.
    #[arg(
        global = true,
        long,
        name = "MODE",
        env = ENV_BUF_MODE,
        default_value_t = BufMode::default(),
        verbatim_doc_comment,
        hide_default_value = true,
    )]
    pub buf_mode: BufMode,

    /// Size of a buffer used for IO operations.
    ///
    /// Smaller values will reduce memory consumption but could negatively affect througput.
    ///
    /// Larger values will increase memory consumption but may improve troughput in some cases.
    ///
    /// Certain commands (which can only operate with whole lines) won't be able to fetch
    /// a line bigger than this limit and will abort their execution instead.
    #[arg(
        global = true,
        long,
        name = "BYTES",
        env = ENV_BUF_SIZE,
        default_value_t = DEFAULT_BUF_SIZE,
    )]
    pub buf_size: usize,
}
