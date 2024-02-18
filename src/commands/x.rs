use super::get_meta;
use crate::args::ENV_BUF_MODE;
use crate::args::ENV_BUF_SIZE;
use crate::args::ENV_NULL;
use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_examples;
use crate::command_meta;
use crate::commands::cat;
use crate::io::LineReader;
use crate::pattern;
use crate::pattern::Expression;
use crate::pattern::ExpressionValue;
use crate::pattern::Item;
use crate::pattern::Pattern;
use crate::pattern::SimpleItem;
use crate::pattern::SimplePattern;
use anyhow::Result;
use bstr::ByteVec;
use clap::builder::OsStr;
use std::env;
use std::env::current_exe;
use std::io::Write;
use std::panic::resume_unwind;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::thread::JoinHandle;
use which::which;

#[cfg(target_family = "windows")]
const DEFAULT_SHELL: &str = "cmd";
#[cfg(not(target_family = "windows"))]
const DEFAULT_SHELL: &str = "sh";

pub const META: Meta = command_meta! {
    name: "x",
    group: Group::Transformers,
    args: Args,
    run: run,
    examples: command_examples! [
        "Empty expression is replaced by input line": {
            args: &["Hello {}!"],
            input: &["first", "second", "third"],
            output: &["Hello first!", "Hello second!", "Hello third!"],
        },
        "Expression with commands to process input line": {
            args: &["Hello {upper}!"],
            input: &["first", "second", "third"],
            output: &["Hello FIRST!", "Hello SECOND!", "Hello THIRD!"],
        },
        "Multiple expressions run as parallel shell pipelines": {
            args: &["{seq}. Hello {upper}!"],
            input: &["first", "second", "third"],
            output: &["1. Hello FIRST!", "2. Hello SECOND!", "3. Hello THIRD!"],
        },
    ],
};

/// Compose parallel shell pipelines using a pattern.
#[derive(clap::Args)]
struct Args {
    /// Output pattern(s).
    ///
    /// Describes how each output line is constructed from the input.
    ///
    /// Multiple patterns are joined together using a space character.
    #[arg(required = true)]
    pattern: Vec<String>,

    /// Escape character for the pattern.
    #[arg(short, long, value_name = "CHAR", default_value_t = '\\')]
    escape: char,

    /// Shell used to evaluate `{# ...}` expressions.
    ///
    /// Default value: `cmd` on Windows, `sh` everywhere else.
    #[arg(short, long, env = "SHELL")]
    shell: Option<String>,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let raw_pattern = args.pattern.join(" ");
    let pattern = Pattern::parse(&raw_pattern, args.escape)?;

    if let Some(pattern) = pattern.try_simplify() {
        return eval_simple_pattern(context, &pattern);
    }

    eval_pattern(context, &pattern, args.shell.as_deref())
}

fn eval_simple_pattern(context: &Context, pattern: &SimplePattern) -> Result<()> {
    let mut reader = context.line_reader();
    let mut writer = context.writer();

    while let Some(line) = reader.read_line()? {
        for item in pattern.items() {
            match item {
                SimpleItem::Constant(value) => writer.write(value.as_bytes())?,
                SimpleItem::Expression => writer.write(line)?,
            }
        }
        writer.write_separator()?;
    }

    Ok(())
}

enum EvalItem {
    Constant(String),
    Reader(LineReader<ChildStdout>),
}

fn eval_pattern(context: &Context, pattern: &Pattern, shell: Option<&str>) -> Result<()> {
    let mut command_builder = CommandBuilder::new(context, shell);
    let mut children = Vec::new();
    let mut items = Vec::new();
    let mut stdins = Vec::new();

    // Build and spawn child process pipelines.
    for item in pattern.items() {
        match &item {
            Item::Constant(value) => items.push(EvalItem::Constant(value.clone())),
            Item::Expression(ref expression) => {
                let (stdin, mut new_children, stdout) =
                    command_builder.build_expression(expression)?;

                children.append(&mut new_children);
                items.push(EvalItem::Reader(context.line_reader_from(stdout)));

                if stdin.is_some() {
                    stdins.push(stdin);
                }
            }
        }
    }

    // "reader" thread which distributes main process stdin to all child processes.
    let thread_context = context.clone();
    let thread: JoinHandle<Result<()>> = thread::spawn(move || {
        if stdins.iter().all(Option::is_none) {
            return Ok(()); // None of child proceses use stdin.
        }

        let mut reader = thread_context.chunk_reader();

        while let Some(chunk) = reader.read_chunk()? {
            for stdin in &mut stdins {
                if let Some(reader) = stdin {
                    if let Err(err) = reader.write_all(chunk) {
                        if err.kind() == std::io::ErrorKind::BrokenPipe {
                            // Do not end the whole thread just because one child process ended.
                            // Keep writing data to the other child processes which are still running.
                            stdin.take();
                        } else {
                            return Err(err.into());
                        }
                    }
                }
            }

            if stdins.iter().all(Option::is_none) {
                break; // All child stdins are closed.
            }
        }

        Ok(())
    });

    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    // Compose output lines from child processes stdout.
    'outer: loop {
        for item in &mut items {
            match item {
                EvalItem::Constant(value) => buffer.push_str(value),
                EvalItem::Reader(reader) => {
                    if let Some(line) = reader.read_line()? {
                        buffer.push_str(line);
                    } else {
                        break 'outer; // Quit as soon as one of child processes ends.
                    }
                }
            }
        }
        writer.write_line(&buffer)?;
        buffer.clear();
    }

    // Make sure all child processes are terminated.
    // This will cause the "reader" thread to end by detecting "broken pipes".
    for mut child in children {
        if child.try_wait()?.is_none() {
            child.kill()?;
        }
    }

    // Try to wait for the "reader" thread to finish (non-blockingly).
    if thread.is_finished() {
        return match thread.join() {
            Ok(res) => res,
            Err(err) => resume_unwind(err),
        };
    }

    // At this moment, the "reader" thread is blocked on read from stdin.
    // There is no way how to interrupt it, so we just let the thread die alongside the main process.
    // Reimplementing this with async Rust is probably not worth the effort, because:
    // 1. It only happens during interactive usage when stdin is TTY.
    // 2. And all process pipelines must contain at least one process which does not open stdin.
    Ok(())
}

struct CommandBuilder<'a> {
    context: &'a Context,
    shell: Option<&'a str>,
    stdbuf_path: Option<which::Result<PathBuf>>,
}

impl<'a> CommandBuilder<'a> {
    fn new(context: &'a Context, shell: Option<&'a str>) -> Self {
        Self {
            context,
            shell,
            stdbuf_path: None,
        }
    }

    fn build_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<(Option<ChildStdin>, Vec<Child>, ChildStdout)> {
        match &expr.value {
            ExpressionValue::RawShell(command) => self.build_raw_shell(command, expr.no_stdin),
            ExpressionValue::Pipeline(commands) => self.build_pipeline(commands, expr.no_stdin),
        }
    }

    fn build_raw_shell(
        &self,
        sh_command: &str,
        no_stdin: bool,
    ) -> Result<(Option<ChildStdin>, Vec<Child>, ChildStdout)> {
        let shell = self.shell.unwrap_or(DEFAULT_SHELL);
        let mut command = Command::new(shell);

        if Path::new(shell).file_stem() == Some(&OsStr::from("cmd")) {
            command.arg("/c");
        } else {
            command.arg("-c");
        }

        command.arg(sh_command);
        command.stdout(Stdio::piped());

        if no_stdin {
            command.stdin(Stdio::null());
        } else {
            command.stdin(Stdio::piped());
        }

        let mut child = command.spawn()?;
        let stdin = child.stdin.take();

        let stdout = child
            .stdout
            .take()
            .expect("Could not get ChildStdout from raw shell");

        Ok((stdin, vec![child], stdout))
    }

    fn build_pipeline(
        &mut self,
        commands: &[pattern::Command],
        mut no_stdin: bool,
    ) -> Result<(Option<ChildStdin>, Vec<Child>, ChildStdout)> {
        let mut children = Vec::new();
        let mut stdin = None;
        let mut stdout = None;

        for params in commands {
            let (mut command, group) = self.build(params)?;

            if group == Group::Generators {
                command.stdin(Stdio::null());
                no_stdin = true;
            } else if let Some(stdout) = stdout {
                command.stdin(Stdio::from(stdout));
            } else {
                command.stdin(Stdio::piped());
            }

            command.stdout(Stdio::piped());
            let mut child = command.spawn()?;

            if no_stdin {
                stdin = None;
            } else if stdin.is_none() {
                stdin = child.stdin.take(); // The first process in pipeline
            }

            stdout = child.stdout.take();
            children.push(child);
        }

        if stdout.is_none() {
            let mut child = self
                .build_default_internal()?
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            stdin = child.stdin.take();
            stdout = child.stdout.take();
        }

        let stdin = stdin.take();
        let stdout = stdout
            .take()
            .expect("Could not get ChildStdout from pipeline");

        Ok((stdin, children, stdout))
    }

    fn build(&mut self, params: &pattern::Command) -> Result<(Command, Group)> {
        let pattern::Command {
            name,
            args,
            external,
        } = params;

        if !external {
            if let Some(meta) = get_meta(name) {
                let command = self.build_internal(Some(name), args)?;
                return Ok((command, meta.group));
            }
            if name == env!("CARGO_PKG_NAME") {
                if let Some((name, args)) = args.split_first() {
                    if let Some(meta) = get_meta(name) {
                        let command = self.build_internal(Some(name), args)?;
                        return Ok((command, meta.group));
                    }
                }
                let command = self.build_internal(None, args)?;
                return Ok((command, Group::Transformers));
            }
        }

        if self.context.buf_mode().is_line() {
            if let Ok(stdbuf) = self.stdbuf() {
                let mut command = Command::new(stdbuf);
                command.arg("-oL"); // Output line buffering
                command.arg(name);
                command.args(args);
                return Ok((command, Group::Transformers));
            }
        }

        let mut command = Command::new(name);
        command.args(args);
        Ok((command, Group::Transformers))
    }

    fn build_internal(&self, name: Option<&str>, args: &[String]) -> Result<Command> {
        let mut command = Command::new(current_exe()?);

        command.env(ENV_NULL, self.context.separator().is_null().to_string());
        command.env(ENV_BUF_MODE, self.context.buf_mode().to_string());
        command.env(ENV_BUF_SIZE, self.context.buf_size().to_string());

        if let Some(name) = name {
            command.arg(name);
        }

        command.args(args);
        Ok(command)
    }

    fn build_default_internal(&self) -> Result<Command> {
        self.build_internal(Some(cat::META.name), &[])
    }

    fn stdbuf(&mut self) -> &which::Result<PathBuf> {
        self.stdbuf_path.get_or_insert_with(|| which("stdbuf"))
    }
}
