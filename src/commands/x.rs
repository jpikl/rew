use super::get_meta;
use crate::command::Context;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::commands::cat;
use crate::io::LineReader;
use crate::pattern;
use crate::pattern::Item;
use crate::pattern::Pattern;
use crate::pattern::SimpleItem;
use crate::pattern::SimplePattern;
use anyhow::Result;
use bstr::ByteVec;
use std::env::current_exe;
use std::io::Write;
use std::panic::resume_unwind;
use std::process::Child;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::thread::JoinHandle;

pub const META: Meta = command_meta! {
    name: "x",
    group: Group::Transformers,
    args: Args,
    run: run,
};

/// Compose parallel shell pipelines using a pattern.
#[derive(clap::Args)]
struct Args {
    /// Composition pattern.
    ///
    /// `abc`             Constant  
    /// `{}`              Empty expression     
    /// `{cmd}`           Expression with a filter command
    /// `{cmd a b}`       Expression with a filter command and args
    /// `{x|y a b|z}`     Expression with a command pipeline
    /// `a{}b{x|y a b}c`  Mixed constant and expresions.
    #[arg(verbatim_doc_comment)]
    pattern: String,

    /// Escape character for the pattern.
    #[arg(short, long, value_name = "CHAR", default_value_t = '\\')]
    escape: char,
}

fn run(context: &Context, args: &Args) -> Result<()> {
    let pattern = Pattern::parse(&args.pattern, args.escape)?;

    if let Some(pattern) = pattern.try_simplify() {
        return eval_simple_pattern(context, &pattern);
    }

    eval_pattern(context, &pattern)
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

fn eval_pattern(context: &Context, pattern: &Pattern) -> Result<()> {
    let mut children = Vec::new();
    let mut items = Vec::new();
    let mut stdins = Vec::new();

    // Build and spawn child process pipelines.
    for item in pattern.items() {
        match &item {
            Item::Constant(value) => items.push(EvalItem::Constant(value.clone())),
            Item::Expression(ref commands) => {
                let (stdin, mut new_children, stdout) = build_command_pipeline(commands)?;
                children.append(&mut new_children);
                items.push(EvalItem::Reader(context.line_reader_from(stdout)));
                stdins.push(Some(stdin));
            }
        }
    }

    // "reader" thread which distributes main process stdin to all child processes.
    let thread_context = context.clone();
    let thread: JoinHandle<Result<()>> = thread::spawn(move || {
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

    // Compose output lines from constant parts and child processes stdout.
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
    // This will cause the "reader" thread to end by detecting "broken pipe" everywhere.
    for mut child in children {
        if child.try_wait()?.is_none() {
            child.kill()?;
        }
    }

    // Try wait for the "reader" thread to finish (non-blockingly)
    if thread.is_finished() {
        return match thread.join() {
            Ok(res) => res,
            Err(err) => resume_unwind(err),
        };
    }

    // At this moment, the "reader" thread is blocked on read from stdin.
    // There is no way how to interrupt it, so we just let the thread die alongside the main process.
    // Reimplementing this with async Rust is probably not worth the effort, because:
    // 1. It only happens during interactive usage (when stdin is TTY).
    // 2. And all child process pipelines must contain at least one process which does not read from stdin.
    Ok(())
}

fn build_command_pipeline(
    commands: &[pattern::Command],
) -> Result<(ChildStdin, Vec<Child>, ChildStdout)> {
    let mut children = Vec::new();
    let mut stdin = None;
    let mut stdout = None;

    for command in commands {
        let mut child = if let Some(stdout) = stdout {
            build_command(command)?
                .stdin(Stdio::from(stdout))
                .stdout(Stdio::piped())
                .spawn()?
        } else {
            build_command(command)?
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?
        };

        if stdin.is_none() {
            stdin = child.stdin.take();
        }

        stdout = child.stdout.take();
        children.push(child);
    }

    if stdin.is_none() && stdout.is_none() {
        let mut child = build_default_internal_command()?
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        stdin = child.stdin.take();
        stdout = child.stdout.take();
    }

    let stdin = stdin
        .take()
        .expect("Could not get ChildStdin from command pipeline");

    let stdout = stdout
        .take()
        .expect("Could not get ChildStdout from command pipeline");

    Ok((stdin, children, stdout))
}

fn build_command(params: &pattern::Command) -> Result<Command> {
    if !params.external {
        for meta in get_meta() {
            if meta.name == params.name {
                return build_internal_command(&params.name, &params.args);
            }
        }
    }

    // TODO: try to wrap using stdbuf to set buffering
    let mut command = Command::new(&params.name);
    command.args(&params.args);
    Ok(command)
}

fn build_internal_command(name: &str, args: &[String]) -> Result<Command> {
    let mut args = args.to_vec();
    args.insert(0, name.to_owned());

    // TODO: pass null/buff config through env
    let mut command = Command::new(current_exe()?);
    command.args(&args);
    Ok(command)
}

fn build_default_internal_command() -> Result<Command> {
    build_internal_command(cat::META.name, &[])
}
