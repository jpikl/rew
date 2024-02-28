use anstream::stream::IsTerminal;
use anyhow::format_err;
use anyhow::Context;
use anyhow::Result;
use std::io::stdout;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use which::which;

pub fn open() -> Result<Option<Child>> {
    if !stdout().is_terminal() {
        return Ok(None);
    }
    // We could eventually do something more complex, such as parsing PAGER
    // env variable like `bat` does https://github.com/sharkdp/bat/issues/158,
    // but that would be an overkill for our use case.
    if let Ok(path) = which("less") {
        // F = Exit immediately if the text fits the entire screen.
        // I = Ignore case when searching.
        // r = Causes "raw" control characters to be displayed.
        // X = Disables sending the termcap (in)itialization.
        return spawn(&path, &["-FIrX"]).map(Some);
    }
    if let Ok(path) = which("more") {
        return spawn(&path, &[]).map(Some);
    }
    Ok(None)
}

fn spawn(path: &Path, args: &[&str]) -> Result<Child> {
    let mut cmd = Command::new(path);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.spawn()
        .with_context(|| format_err!("could not spawn command [{cmd:?}]"))
}
