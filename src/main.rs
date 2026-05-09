//! kairn — a TUI IDE oriented around Kiro AI.

use std::path::PathBuf;
use std::process;

use clap::Parser;

mod app;
mod buffer;
mod commands;
mod config;
mod content_search;
mod desktop;
mod editor;
mod git;
mod kiro;
mod lsp;
mod nav;
mod runner;
mod status;
mod views;
mod types;

/// A TUI IDE oriented around Kiro AI.
#[derive(Parser)]
#[command(name = "kairn", version, about)]
struct Cli {
    /// Directory or file to open.
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let root = if cli.path.is_file() {
        cli.path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        cli.path.clone()
    };

    let mut app = app::App::new(&root);

    if let Err(e) = app.run() {
        eprintln!("kairn: {e}");
        process::exit(1);
    }
}
