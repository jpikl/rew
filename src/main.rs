use rew::app;
use rew::commands::get_meta;
use rew::commands::METAS;
use rew::error::Reporter;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let app = app::build(&METAS);
    let reporter = Reporter::new(&app);

    let matches = match app.try_get_matches() {
        Ok(matches) => matches,
        Err(error) => {
            reporter.print_args_error(&error);
            return ExitCode::from(2);
        }
    };

    let (cmd_name, cmd_matches) = matches.subcommand().expect("command not matched");
    let cmd = get_meta(cmd_name).expect("command not found");

    match cmd.run(cmd_matches) {
        Ok(()) => ExitCode::from(0),
        Err(error) if is_broken_pipe(&error) => ExitCode::from(0),
        Err(error) => {
            reporter.print_run_error(&error);
            ExitCode::from(1)
        }
    }
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|io_err| io_err.kind() == io::ErrorKind::BrokenPipe)
    })
}
