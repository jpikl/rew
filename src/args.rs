use crate::io::BufModeConfig;
use crate::io::BufSizeConfig;
use crate::io::LineConfig;
use crate::io::LineSeparator;
use clap::Args;
use clap::ValueEnum;
use derive_more::Display;
use std::io::stdout;
use std::io::IsTerminal;

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library https://github.com/Freaky/rust-linereader.
pub const DEFAULT_BUF_SIZE: usize = 32 * 1024;

#[derive(Clone, ValueEnum, Display, Debug, PartialEq, Eq)]
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
    #[arg(global = true, short = '0', long, env = "REW_NULL")]
    null: bool,

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
        env = "REW_BUF_MODE",
        default_value_t = BufMode::default(),
        verbatim_doc_comment,
        hide_default_value = true,
    )]
    buf_mode: BufMode,

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
        env = "REW_BUF_SIZE",
        default_value_t = DEFAULT_BUF_SIZE,
    )]
    buf_size: usize,
}

impl LineConfig for GlobalArgs {
    fn line_separator(&self) -> LineSeparator {
        if self.null {
            LineSeparator::Null
        } else {
            LineSeparator::Newline
        }
    }
}

impl BufModeConfig for GlobalArgs {
    fn buf_full(&self) -> bool {
        self.buf_mode == BufMode::Full
    }
}

impl BufSizeConfig for GlobalArgs {
    fn buf_size(&self) -> usize {
        self.buf_size
    }
}
