use crate::io::LineConfig;
use crate::io::LineReaderConfig;
use crate::io::LineSeparator;
use crate::io::WriterConfig;
use crate::io::OPTIMAL_IO_BUF_SIZE;
use clap::Args;
use clap::ValueEnum;
use derive_more::Display;
use std::io::stdout;
use std::io::IsTerminal;

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

    /// Output buffering.
    ///
    /// - `line` emits output after each new-line character (for interactive usage).
    /// - `full` emits output once the output buffer is full (for maximal throughput).
    ///
    /// Defaults to `line` when stdout is TTY, otherwise is `full`.
    #[arg(
        global = true,
        long,
        env = "REW_BUFF",
        default_value_t = BufMode::default(),
        verbatim_doc_comment,
        hide_default_value = true,
    )]
    buff: BufMode,

    /// Maximum size of an input line (in bytes).
    ///
    /// Attempt to process a longer input line will abort the execution.
    #[arg(
        global = true,
        long,
        name = "BYTES",
        env = "REW_MAX_LINE",
        default_value_t = OPTIMAL_IO_BUF_SIZE,
        verbatim_doc_comment,
    )]
    max_line: usize,
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

impl LineReaderConfig for GlobalArgs {
    fn line_buf_size(&self) -> usize {
        self.max_line
    }
}

impl WriterConfig for GlobalArgs {
    fn write_is_buffered(&self) -> bool {
        match self.buff {
            BufMode::Line => false,
            BufMode::Full => true,
        }
    }
}
