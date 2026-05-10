//! kairn — TUI IDE entry point.

use std::path::PathBuf;

use clap::Parser;
use txv_core::program::Program;
use txv_render::backend::CrosstermBackend;
use txv_render::color::detect_color_mode;

use kairn::completer::AppCompleter;
use kairn::handler::{build_desktop, handle_command, AppState};
use kairn::status::build_status_bar;

#[derive(Parser)]
#[command(name = "kairn", about = "TUI IDE")]
struct Cli {
    /// Directory to open
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Log file (default: .kairn.log)
    #[arg(short = 'l', long = "log", default_value = ".kairn.log")]
    log_file: PathBuf,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short = 'L', long = "log-level", default_value = "info")]
    log_level: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root_dir = std::fs::canonicalize(&cli.path)?;

    // Init logging to file
    let log_file = std::fs::File::create(&cli.log_file)?;
    env_logger::Builder::new()
        .filter_level(cli.log_level.parse().unwrap_or(log::LevelFilter::Info))
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    // Build desktop
    let desktop = build_desktop(&root_dir);

    // Build status bar
    let status = build_status_bar(Box::new(AppCompleter::new(root_dir.clone())), 60, root_dir.clone());

    // Build program
    let mut program = Program::new(Box::new(status), Box::new(desktop));

    // App state
    let mut state = AppState::new(root_dir);

    // Run
    let color_mode = detect_color_mode();
    let mut backend = CrosstermBackend::new(color_mode);

    program.run(&mut backend, |ctx| {
        handle_command(ctx, &mut state);
    });

    Ok(())
}
