use crate::args::GlobalArgs;
use crate::command::Group;
use crate::command::Meta;
use crate::command_meta;
use crate::io::Writer;
use anyhow::Result;
use std::ffi::OsString;

pub const META: Meta = command_meta! {
    name: "stream",
    group: Group::Generators,
    args: Args,
    run: run,
};

/// Print arguments as lines.
#[derive(clap::Args)]
struct Args {
    /// Values to print.
    values: Vec<OsString>,
}

fn run(global_args: &GlobalArgs, args: &Args) -> Result<()> {
    let mut writer = Writer::from_stdout(global_args);

    for value in &args.values {
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::ffi::OsStrExt;
            writer.write_line(value.as_bytes())?;
        }
        #[cfg(target_family = "windows")]
        {
            writer.write_line(value.to_string_lossy().as_bytes())?;
        }
    }

    Ok(())
}
