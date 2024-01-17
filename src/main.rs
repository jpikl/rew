use anyhow::Error;
use anyhow::Result;
use bstr::ByteSlice;
use bstr::ByteVec;
use clap::command;
use clap::ArgMatches;
use clap::Args;
use clap::Command;
use clap::FromArgMatches;
use clap::ValueEnum;
use linereader::LineReader;
use memchr::memchr;
use memchr::memrchr;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::IsTerminal;
use std::io::Read;
use std::io::StdinLock;
use std::io::StdoutLock;
use std::io::Write;
use thiserror::Error;
use unidecode::unidecode_char;

fn main() {
    handle_error(run().or_else(ignore_broken_pipe));
}

fn handle_error(result: Result<()>) {
    if let Err(error) = result {
        report_error(error, &mut anstream::stderr()).expect("Failed to write error to stderr!");
        std::process::exit(1);
    }
}

fn report_error(error: Error, stderr: &mut impl Write) -> io::Result<()> {
    writeln!(stderr, "{}: {}", "error".red().bold(), error)?;

    for cause in error.chain().skip(1) {
        writeln!(stderr, "{}: {}", "cause".red(), cause)?;
    }

    Ok(())
}

fn ignore_broken_pipe(error: Error) -> Result<()> {
    for cause in error.chain() {
        if let Some(io_error) = cause.downcast_ref::<io::Error>() {
            if io_error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
        }
    }
    Err(error)
}

fn run() -> Result<()> {
    let commands = get_commands();
    let app = build_app(&commands);

    if let Some((name, matches)) = app.get_matches().subcommand() {
        for command in &commands {
            if name == command.name {
                return (command.run)(matches);
            }
        }
    }

    unreachable!("clap should handle missing or invalid command");
}

fn get_commands() -> Vec<&'static CommandMeta> {
    vec![&ASCII, &FIRST, &NORM, &TRIM]
}

fn build_app(commands: &Vec<&'static CommandMeta>) -> Command {
    let mut app = command!("rew").subcommand_required(true);

    for command in commands {
        app = app.subcommand((command.build)())
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

struct CommandMeta {
    name: &'static str,
    build: fn() -> Command,
    run: fn(&ArgMatches) -> Result<()>,
}

macro_rules! command_meta {
    (name: $name:literal, args: $args:ident, run: $run:ident,) => {
        CommandMeta {
            name: $name,
            build: || -> clap::Command { $args::augment_args(Command::new($name)) },
            run: |matches| -> anyhow::Result<()> {
                let global_args = $crate::GlobalArgs::from_arg_matches(matches)?;
                let args = $args::from_arg_matches(matches)?;
                $run(global_args, args)
            },
        }
    };
}

#[derive(Args, Default, Debug, Clone, Eq, PartialEq)]
struct GlobalArgs {
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
    /// Attempt to process a longer input line will abort the execution with error.
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

const NORM: CommandMeta = command_meta! {
    name: "norm",
    args: NormArgs,
    run: norm,
};

/// Normalize line separators to LF
#[derive(Args)]
struct NormArgs;

fn norm(global_args: GlobalArgs, _args: NormArgs) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);

    reader.for_each_line(|line| {
        writer.write_line(line)?;
        Ok(Processing::Continue)
    })
}

const TRIM: CommandMeta = command_meta! {
    name: "trim",
    args: TrimArgs,
    run: trim,
};

/// Trim whitespaces from each line.
///
/// By default, both the beginning and the end are trimmed.
#[derive(Args)]
struct TrimArgs {
    /// Trim the beginning.
    #[arg(short, long)]
    start: bool,

    /// Trim the end.
    #[arg(short, long)]
    end: bool,
}

fn trim(global_args: GlobalArgs, args: TrimArgs) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);

    reader.for_each_line(|line| {
        let result = match (args.start, args.end) {
            (true, true) | (false, false) => line.trim(),
            (true, false) => line.trim_start(),
            (false, true) => line.trim_end(),
        };
        writer.write_line(result)?;
        Ok(Processing::Continue)
    })
}

const ASCII: CommandMeta = command_meta! {
    name: "ascii",
    args: AsciiArgs,
    run: ascii,
};

/// Convert characters to ASCII.
#[derive(Args)]
struct AsciiArgs {
    /// Delete non-ASCII characters instead of converting them.
    #[arg(short, long)]
    delete: bool,
}

fn ascii(global_args: GlobalArgs, args: AsciiArgs) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);
    let mut buffer = Vec::with_capacity(OPTIMAL_IO_BUF_SIZE);

    reader.for_each_block(|block| {
        // Copying chars to buffer is faster then directly writing them to output
        if args.delete {
            block
                .chars()
                .filter(|char| char.is_ascii())
                .for_each(|char| buffer.push(char as u8));
        } else {
            block
                .chars()
                .map(unidecode_char)
                .for_each(|str| buffer.push_str(str));
        }
        writer.write_block(&buffer)?;
        buffer.clear();
        Ok(Processing::Continue)
    })
}

const FIRST: CommandMeta = command_meta! {
    name: "first",
    args: FirstArgs,
    run: first,
};

/// Output first N input line(s).
#[derive(Args)]
struct FirstArgs {
    /// Number of lines to print.
    #[arg(default_value_t = 1)]
    count: u128,
}

fn first(global_args: GlobalArgs, args: FirstArgs) -> Result<()> {
    let mut reader = Reader::from(&global_args);
    let mut writer = Writer::from(&global_args);
    let mut count = args.count;

    if count == 0 {
        return Ok(());
    }

    reader.for_each_line(|line| {
        writer.write_line(line)?;
        count -= 1;

        if count > 0 {
            Ok(Processing::Continue)
        } else {
            Ok(Processing::Abort)
        }
    })
}

// Optimal value for max IO throughput, according to https://www.evanjones.ca/read-write-buffer-size.html
// Also confirmed by some custom benchmarks.
// Also used internally by the `linereader` library.
const OPTIMAL_IO_BUF_SIZE: usize = 32 * 1024;

#[derive(Debug, Error, PartialEq)]
#[error("cannot process input line bigger than '{}' bytes", .0)]
struct MaxLineError(usize);

#[derive(Clone, Default, Debug, PartialEq, Eq)]
enum Separator {
    #[default]
    Newline,
    Null,
}

impl From<&GlobalArgs> for Separator {
    fn from(args: &GlobalArgs) -> Self {
        match args.null {
            false => Self::Newline,
            true => Self::Null,
        }
    }
}

impl Separator {
    fn as_byte(&self) -> u8 {
        match self {
            Self::Newline => b'\n',
            Self::Null => b'\0',
        }
    }

    fn trim_end<'a>(&self, mut line: &'a [u8]) -> &'a [u8] {
        match self {
            Self::Newline => {
                if line.last_byte() == Some(b'\n') {
                    line = &line[..line.len() - 1];
                    if line.last_byte() == Some(b'\r') {
                        line = &line[..line.len() - 1];
                    }
                }
            }
            Self::Null => {
                if line.last_byte() == Some(b'\0') {
                    line = &line[..line.len() - 1];
                }
            }
        }
        line
    }
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
enum Buffering {
    Line,
    Full,
}

impl Display for Buffering {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Line => write!(fmt, "line"),
            Self::Full => write!(fmt, "full"),
        }
    }
}

impl Default for Buffering {
    fn default() -> Self {
        if io::stdout().is_terminal() {
            Self::Line
        } else {
            Self::Full
        }
    }
}

enum Processing {
    Continue,
    Abort,
}

struct Reader<R> {
    inner: R,
    separator: Separator,
    max_line: usize,
}

impl From<&GlobalArgs> for Reader<StdinLock<'static>> {
    fn from(args: &GlobalArgs) -> Self {
        Self::new(io::stdin().lock(), Separator::from(args), args.max_line)
    }
}

impl<R: Read> Reader<R> {
    fn new(inner: R, separator: Separator, max_line: usize) -> Self {
        Self {
            inner,
            separator,
            max_line,
        }
    }

    fn for_each_line<F: FnMut(&[u8]) -> Result<Processing>>(
        &mut self,
        mut action: F,
    ) -> Result<()> {
        let byte_separator = self.separator.as_byte();
        let mut reader =
            LineReader::with_delimiter_and_capacity(byte_separator, self.max_line, &mut self.inner);

        while let Some(batch) = reader.next_batch() {
            let mut batch = batch?;

            while let Some(end) = memchr(byte_separator, batch) {
                let (line, next_batch) = batch.split_at(end + 1);

                if line.len() == self.max_line {
                    return Err(MaxLineError(self.max_line).into());
                }

                match action(self.separator.trim_end(line)) {
                    Ok(Processing::Continue) => {}
                    Ok(Processing::Abort) => return Ok(()),
                    Err(err) => return Err(err),
                }

                batch = next_batch;
            }

            if !batch.is_empty() {
                match action(self.separator.trim_end(batch)) {
                    Ok(Processing::Continue) => {}
                    Ok(Processing::Abort) => return Ok(()),
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(())
    }

    fn for_each_block<F: FnMut(&[u8]) -> Result<Processing>>(
        &mut self,
        mut action: F,
    ) -> Result<()> {
        let mut reader = BufReader::new(&mut self.inner);
        let mut buffer = vec![0; OPTIMAL_IO_BUF_SIZE];

        loop {
            let len = reader.read(&mut buffer)?;
            if len == 0 {
                break;
            }
            match action(&buffer[..len]) {
                Ok(Processing::Continue) => {}
                Ok(Processing::Abort) => return Ok(()),
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}

struct Writer<W> {
    inner: W,
    separator: u8,
    buffering: Buffering,
}

impl From<&GlobalArgs> for Writer<BufWriter<StdoutLock<'static>>> {
    fn from(args: &GlobalArgs) -> Self {
        let inner = BufWriter::with_capacity(OPTIMAL_IO_BUF_SIZE, io::stdout().lock());
        Self::new(inner, Separator::from(args), args.buff.clone())
    }
}

impl<W: Write> Writer<W> {
    fn new(inner: W, separator: Separator, buffering: Buffering) -> Self {
        Self {
            inner,
            separator: separator.as_byte(),
            buffering,
        }
    }

    fn write_line(&mut self, line: &[u8]) -> Result<()> {
        self.inner.write_all(line)?;
        self.inner.write_all(&[self.separator])?;

        match self.buffering {
            Buffering::Line => self.inner.flush().map_err(Into::into),
            Buffering::Full => Ok(()),
        }
    }

    fn write_block(&mut self, block: &[u8]) -> Result<()> {
        match self.buffering {
            Buffering::Line => {
                // We do not care much about the performance in this mode
                if let Some(pos) = memrchr(self.separator, block) {
                    let (before, after) = block.split_at(pos + 1);
                    self.inner.write_all(before)?;
                    self.inner.flush()?;
                    self.inner.write_all(after)?;
                } else {
                    self.inner.write_all(block)?;
                }
            }
            Buffering::Full => self.inner.write_all(block)?,
        }
        Ok(())
    }
}
