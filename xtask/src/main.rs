mod command;
mod docs;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use command::Adapter;
use docs::generate_reference;
use docs::generate_summary;
use rew::app::build_app;
use rew::app::handle_error;
use rew::commands::METAS;
use std::env;
use std::fs::create_dir_all;
use std::path::Path;

/// Execute project task.
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub task: Task,
}

#[derive(Subcommand)]
pub enum Task {
    /// Generate markdown documentation.
    #[command()]
    GenDocs,
    /// Generate man pages.
    #[command()]
    GenMan,
}

fn main() {
    handle_error(run());
}

fn run() -> Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    let root_path = Path::new(&manifest_dir)
        .parent()
        .ok_or_else(|| anyhow!("'{}' does not have parent", manifest_dir))?;

    let cli = Cli::parse();

    let mut app = build_app(&METAS);
    app.build();

    match cli.task {
        Task::GenDocs => {
            let docs_path = root_path.join("docs");
            let reference_path = docs_path.join("reference");
            let summary_path = docs_path.join("SUMMARY.md");

            if !reference_path.exists() {
                create_dir_all(&reference_path)?;
            }

            let adapter = Adapter::new(&app);
            generate_summary(&adapter, &summary_path)?;
            generate_reference(&adapter, &reference_path)?;
        }
        Task::GenMan => unimplemented!(),
    }

    Ok(())
}
