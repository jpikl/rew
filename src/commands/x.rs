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
    let mut stdins = Vec::new();
    let mut items = Vec::new();

    for item in pattern.items() {
        match &item {
            Item::Constant(value) => items.push(EvalItem::Constant(value.clone())),
            Item::Expression(ref commands) => {
                let (stdin, stdout) = build_command_pipeline(commands)?;
                stdins.push(stdin);
                items.push(EvalItem::Reader(context.line_reader_from(stdout)));
            }
        }
    }

    let thread_context = context.clone();
    let thread_result: JoinHandle<Result<()>> = thread::spawn(move || {
        let mut reader = thread_context.chunk_reader();

        while let Some(chunk) = reader.read_chunk()? {
            for mut stdin in &stdins {
                stdin.write_all(chunk)?;
            }
        }

        Ok(())
    });

    let mut writer = context.writer();
    let mut buffer = context.uninit_buf();

    'outer: loop {
        for item in &mut items {
            match item {
                EvalItem::Constant(value) => buffer.push_str(value),
                EvalItem::Reader(reader) => {
                    if let Some(line) = reader.read_line()? {
                        buffer.push_str(line);
                    } else {
                        break 'outer;
                    }
                }
            }
        }

        writer.write_line(&buffer)?;
        buffer.clear();
    }

    match thread_result.join() {
        Ok(res) => res,
        Err(err) => resume_unwind(err),
    }
}

fn build_command_pipeline(commands: &[pattern::Command]) -> Result<(ChildStdin, ChildStdout)> {
    let mut stdin: Option<ChildStdin> = None;
    let mut stdout: Option<ChildStdout> = None;

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

    Ok((stdin, stdout))
}

fn build_command(params: &pattern::Command) -> Result<Command> {
    if !params.external {
        for meta in get_meta() {
            if meta.name == params.name {
                return build_internal_command(&params.name, &params.args);
            }
        }
    }

    let mut command = Command::new(&params.name);
    command.args(&params.args);
    Ok(command)
}

fn build_internal_command(name: &str, args: &[String]) -> Result<Command> {
    let mut args = args.to_vec();
    args.insert(0, name.to_owned());

    let mut command = Command::new(current_exe()?);
    command.args(&args);
    Ok(command)
}

fn build_default_internal_command() -> Result<Command> {
    build_internal_command(cat::META.name, &[])
}
