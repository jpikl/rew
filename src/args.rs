use crate::io::Buffering;
use crate::io::Reader;
use crate::io::Separator;
use crate::io::Writer;
use crate::io::OPTIMAL_IO_BUF_SIZE;
use clap::Args;
use std::io;
use std::io::BufWriter;
use std::io::StdinLock;
use std::io::StdoutLock;

#[allow(clippy::module_name_repetitions)]
#[derive(Args, Default, Debug, Clone, Eq, PartialEq)]
pub struct GlobalArgs {
    /// Line delimiter is NUL, not newline.
    #[arg(global = true, short = '0', long, env = "REW_NULL")]
    null: bool,

    /// Output buffering.
    ///
    /// - `line` emits output after processing each line.
    /// - `full` emits output once the output buffer is full (ensuring max possible throughput).
    ///
    /// Defaults to `line` when the output is TTY, otherwise is `full`.
    #[arg(
        global = true,
        long,
        env = "REW_BUFF",
        default_value_t = Buffering::default(),
        verbatim_doc_comment,
        hide_default_value = true,
    )]
    buff: Buffering,

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

impl From<&GlobalArgs> for Separator {
    fn from(args: &GlobalArgs) -> Self {
        if args.null {
            Self::Null
        } else {
            Self::Newline
        }
    }
}

impl From<&GlobalArgs> for Reader<StdinLock<'static>> {
    fn from(args: &GlobalArgs) -> Self {
        Self::new(io::stdin().lock(), Separator::from(args), args.max_line)
    }
}

impl From<&GlobalArgs> for Writer<BufWriter<StdoutLock<'static>>> {
    fn from(args: &GlobalArgs) -> Self {
        let inner = BufWriter::with_capacity(OPTIMAL_IO_BUF_SIZE, io::stdout().lock());
        Self::new(inner, Separator::from(args).as_byte(), args.buff.clone())
    }
}
