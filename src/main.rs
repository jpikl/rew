use anyhow::Error;
use anyhow::Result;
use rew::app::build_app;
use rew::app::handle_error;
use rew::commands::get_meta;
use rew::commands::METAS;

fn main() {
    handle_error(run().or_else(ignore_broken_pipe));
}

fn run() -> Result<()> {
    let app = build_app(&METAS);

    if let Some((name, matches)) = app.get_matches().subcommand() {
        if let Some(meta) = get_meta(name) {
            return (meta.run)(matches);
        }
    }

    unreachable!("clap should handle missing or invalid command");
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
