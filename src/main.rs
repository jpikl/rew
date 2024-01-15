use anyhow::Error;
use anyhow::Result;
use bstr::BStr;
use bstr::BString;
use bstr::ByteSlice;
use bstr::ByteVec;
use clap::command;
use clap::ArgMatches;
use clap::Args;
use clap::Command;
use clap::FromArgMatches;
use clap::ValueEnum;
use linereader::LineReader;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::IsTerminal;
use std::io::StdinLock;
use std::io::StdoutLock;
use std::io::Write;
use thiserror::Error;
use unidecode::unidecode_char;

fn main() {
    handle_error(run().or_else(ignore_broken_pipe));
}

fn ignore_broken_pipe(error: Error) -> Result<()> {
    for cause in error.chain() {
        if let Some(io_error) = cause.downcast_ref::<std::io::Error>() {
            if io_error.kind() == std::io::ErrorKind::BrokenPipe {
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

pub struct CommandMeta {
    name: &'static str,
    build: fn() -> Command,
    run: fn(&ArgMatches) -> Result<()>,
}

const ASCII: CommandMeta = command_meta! {
    name: "ascii",
    args: AsciiArgs,
    run: ascii,
};

const TRIM: CommandMeta = command_meta! {
    name: "trim",
    args: TrimArgs,
    run: trim,
};

const NORM: CommandMeta = command_meta! {
    name: "norm",
    args: NormArgs,
    run: norm,
};

pub fn get_commands() -> Vec<&'static CommandMeta> {
    vec![&ASCII, &NORM, &TRIM]
}

pub fn build_app(commands: &Vec<&'static CommandMeta>) -> Command {
    let mut app = command!("rew").subcommand_required(true);

    for command in commands {
        app = app.subcommand((command.build)())
    }

    GlobalArgs::augment_args(app.next_help_heading("Global options"))
}

pub fn handle_error(result: Result<()>) {
    if let Err(error) = result {
        report_error(error, &mut anstream::stderr()).expect("Failed to write error to stderr!");
        std::process::exit(1);
    }
}

fn report_error(error: Error, stderr: &mut impl Write) -> std::io::Result<()> {
    writeln!(stderr, "{}: {}", "error".red().bold(), error)?;

    for cause in error.chain().skip(1) {
        writeln!(stderr, "{}: {}", "cause".red(), cause)?;
    }

    Ok(())
}

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
enum Buffering {
    Line,
    Block,
}

impl Display for Buffering {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Line => write!(fmt, "line"),
            Self::Block => write!(fmt, "block"),
        }
    }
}

impl Default for Buffering {
    fn default() -> Self {
        if std::io::stdout().is_terminal() {
            Self::Line
        } else {
            Self::Block
        }
    }
}

#[macro_export]
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
    /// - `line` will flush output buffer after each line (default in interactive mode).
    /// - `block` will flush output buffer when full (default in batch mode).
    #[arg(
        global = true,
        long,
        env = "REW_BUFF",
        default_value_t = Buffering::default(),
        verbatim_doc_comment,
        hide_default_value = true,
    )]
    buff: Buffering,

    /// Maximum size of an input line.
    ///
    /// Attempt to process a bigger input line will result in an error and immediate termination.
    /// Accepts values like `1024`, `1K`, `2MiB`, etc.
    /// Using `0` as a value will disable the limit.
    #[arg(
        global = true,
        long,
        name = "BYTES",
        env = "REW_MAX_LINE",
        default_value_t = DEFAULT_CAPACITY,
        verbatim_doc_comment,
    )]
    max_line: u64,
}

/// Trim whitespaces from each line.
///
/// By default, both the beginning and the end are trimmed.
#[derive(Args)]
struct TrimArgs {
    /// Trim the beginning
    #[arg(short, long)]
    start: bool,

    /// Trim the end
    #[arg(short, long)]
    end: bool,
}

fn trim(global_args: GlobalArgs, args: TrimArgs) -> Result<()> {
    let mut writer = RecordWriter::from(&global_args);

    for_each_record(&global_args, |record| {
        let result = match (args.start, args.end) {
            (true, true) | (false, false) => record.trim(),
            (true, false) => record.trim_start(),
            (false, true) => record.trim_end(),
        };

        writer.write(result)?;
        Ok(true)
    })
}

/// Convert characters to ASCII.
#[derive(Args)]
pub struct AsciiArgs {
    /// Delete non-ASCII characters instead of converting them.
    #[arg(short, long)]
    pub delete: bool,
}

fn ascii(global_args: GlobalArgs, args: AsciiArgs) -> Result<()> {
    let mut writer = RecordWriter::from(&global_args);
    let mut result = create_buffer(global_args.max_line as usize);

    for_each_record(&global_args, |record| {
        if record.is_ascii() {
            writer.write(record)?;
        } else {
            match args.delete {
                true => record
                    .chars()
                    .filter(char::is_ascii)
                    .for_each(|char| result.push_char(char)),
                false => record
                    .chars()
                    .map(unidecode_char)
                    .for_each(|char| result.push_str(char)),
            }
            writer.write(&result)?;
            result.clear();
        }
        Ok(true)
    })
}

/// Norm
#[derive(Args)]
struct NormArgs {}

fn norm(global_args: GlobalArgs, _args: NormArgs) -> Result<()> {
    let mut writer = RecordWriter::from(&global_args);

    for_each_record(&global_args, |record| {
        writer.write(record)?;
        Ok(true)
    })
}

const DEFAULT_CAPACITY: u64 = 1024 * 64;

fn for_each_record<F: FnMut(&BStr) -> Result<bool>>(
    args: &GlobalArgs,
    mut action: F,
) -> Result<()> {
    let mut result = Ok(());
    let capacity = args.max_line as usize;
    if args.null {
        LineReader::with_delimiter_and_capacity(b'\0', capacity, open_stdin()).for_each(|record| {
            let trimmed_record = trim_null(record);
            if trimmed_record.len() == capacity {
                result = Err(MaxLineError(args.max_line).into());
                return Ok(false);
            }
            match action(trimmed_record.as_bstr()) {
                Ok(true) => Ok(true),
                Ok(false) => Ok(false),
                Err(err) => {
                    result = Err(err);
                    Ok(false)
                }
            }
        })
    } else {
        LineReader::with_delimiter_and_capacity(b'\n', capacity, open_stdin()).for_each(|record| {
            let trimmed_record = trim_newline(record);
            if trimmed_record.len() == capacity {
                result = Err(MaxLineError(args.max_line).into());
                return Ok(false);
            }
            match action(trimmed_record.as_bstr()) {
                Ok(true) => Ok(true),
                Ok(false) => Ok(false),
                Err(err) => {
                    result = Err(err);
                    Ok(false)
                }
            }
        })
    }?;
    result
}

fn trim_newline(mut line: &[u8]) -> &[u8] {
    if line.last_byte() == Some(b'\n') {
        line = &line[..line.len() - 1];
        if line.last_byte() == Some(b'\r') {
            line = &line[..line.len() - 1];
        }
    }
    line
}

fn trim_null(mut record: &[u8]) -> &[u8] {
    if record.last_byte() == Some(b'\0') {
        record = &record[..record.len() - 1];
    }
    record
}

fn create_buffer(capacity: usize) -> BString {
    Vec::with_capacity(capacity).into()
}

fn open_stdin() -> BufReader<StdinLock<'static>> {
    BufReader::new(std::io::stdin().lock())
}

fn open_stdout() -> BufWriter<StdoutLock<'static>> {
    BufWriter::new(std::io::stdout().lock())
}

#[derive(Debug, Error, PartialEq)]
#[error("cannot process input line bigger than '{}'", 123)]
struct MaxLineError(u64);

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum Separator {
    #[default]
    Newline,
    Null,
}

impl Separator {
    pub fn as_byte(&self) -> u8 {
        match self {
            Separator::Newline => b'\n',
            Separator::Null => b'\0',
        }
    }
}

impl From<&GlobalArgs> for Separator {
    fn from(args: &GlobalArgs) -> Self {
        match args.null {
            false => Self::Newline,
            true => Self::Null,
        }
    }
}

pub struct RecordWriter<W> {
    writer: W,
    separator: u8,
    buffering: Buffering,
}

impl From<&GlobalArgs> for RecordWriter<BufWriter<StdoutLock<'static>>> {
    fn from(args: &GlobalArgs) -> Self {
        Self::new(
            open_stdout(),
            Separator::from(args).as_byte(),
            args.buff.clone(),
        )
    }
}

impl<W: Write> RecordWriter<W> {
    fn new(writer: W, separator: u8, buffering: Buffering) -> Self {
        Self {
            writer,
            separator,
            buffering,
        }
    }

    fn write(&mut self, value: &[u8]) -> Result<()> {
        self.writer.write_all(value)?;
        self.writer.write_all(&[self.separator])?;

        match self.buffering {
            Buffering::Line => self.writer.flush().map_err(Into::into),
            Buffering::Block => Ok(()),
        }
    }
}
