//! kairn — TUI IDE entry point.

use std::path::PathBuf;

use clap::Parser;
use txv_core::program::Program;
use txv_render::backend::CrosstermBackend;
use txv_render::color::detect_color_mode;

use kairn::completer::AppCompleter;
use kairn::config::load_config;
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
    // Layer 3: Global panic handler — restore terminal before crashing
    std::panic::set_hook(Box::new(|info| {
        // Restore terminal state
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        // Print panic info
        eprintln!("\n\x1b[1;31mkairn panicked!\x1b[0m");
        eprintln!("{info}");
        if let Some(loc) = info.location() {
            eprintln!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
        }
        eprintln!("\nPlease report this bug.");
    }));

    let cli = Cli::parse();

    // Nesting guard: prevent running inside a suspended kairn session
    if std::env::var("KAIRN_SUSPENDED").is_ok() {
        eprintln!("kairn is already running (suspended). Use 'exit' to return.");
        std::process::exit(1);
    }

    let root_dir = std::fs::canonicalize(&cli.path)?;

    // Init logging to file
    let log_file = std::fs::File::create(&cli.log_file)?;
    env_logger::Builder::new()
        .filter_level(cli.log_level.parse().unwrap_or(log::LevelFilter::Info))
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    // Load config
    let settings = load_config(&root_dir);

    // Build desktop
    let desktop = build_desktop(&root_dir);

    // Build status bar
    let status = build_status_bar(
        Box::new(AppCompleter::new(root_dir.clone())),
        settings.clock_interval,
        root_dir.clone(),
    );

    // Build program
    let mut program = Program::new(Box::new(status), Box::new(desktop));

    // App state
    let mut state = AppState::with_settings(root_dir, settings);

    // Run
    let color_mode = detect_color_mode();
    let mut backend = CrosstermBackend::new(color_mode);

    program.run(&mut backend, |ctx| {
        handle_command(ctx, &mut state);
    });

    Ok(())
}
