//! kairn — TUI IDE entry point.

use std::path::PathBuf;

use clap::Parser;
use txv_core::program::Program;
use txv_core::run::Backend;
use txv_render::backend::CrosstermBackend;
use txv_render::color::detect_color_mode;

use kairn::build_desktop::build_desktop;
use kairn::completer::AppCompleter;
use kairn::config::load_config;
use kairn::handler::{handle_command, AppState};
use kairn::session;
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

    /// Run as MCP bridge (stdin↔socket proxy) and exit
    #[arg(long = "mcp-connect")]
    mcp_connect: bool,
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

    // MCP bridge mode: proxy stdin↔socket and exit
    if cli.mcp_connect {
        return kairn::mcp::bridge::run_mcp_bridge().map_err(|e| anyhow::anyhow!("MCP bridge failed: {e}"));
    }

    // Nesting guard: prevent running inside a suspended kairn session
    if std::env::var("KAIRN_SUSPENDED").is_ok() {
        eprintln!("kairn is already running (suspended). Use 'exit' to return.");
        std::process::exit(1);
    }

    let root_dir = std::fs::canonicalize(&cli.path)?;

    // Compute socket path and check instance lock
    let socket_path = kairn::mcp::socket_path::socket_path(&root_dir);
    if std::os::unix::net::UnixStream::connect(&socket_path).is_ok() {
        eprintln!("kairn is already running for this project.");
        std::process::exit(1);
    }

    // Start MCP listener
    let mcp_snapshot = std::sync::Arc::new(std::sync::Mutex::new(kairn::mcp::snapshot::McpSnapshot::default()));
    let mcp_socket = kairn::mcp::listener::start_mcp_listener(std::sync::Arc::clone(&mcp_snapshot), &socket_path);
    if let Ok(ref sock) = mcp_socket {
        kairn::mcp::agent_file::write_agent_file(&root_dir);
        std::env::set_var("KAIRN_MCP_SOCKET", sock.to_string_lossy().as_ref());
    }

    // Init logging to file
    let log_file = std::fs::File::create(&cli.log_file)?;
    env_logger::Builder::new()
        .filter_level(cli.log_level.parse().unwrap_or(log::LevelFilter::Info))
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    // Load config
    let settings = load_config(&root_dir);

    // Load saved session (if any)
    let saved_session = session::load_session(&root_dir);

    // Build desktop
    let git_keys = settings.git_keys.clone();
    let mut app_state = AppState::with_settings(root_dir.clone(), settings);
    app_state.mcp_snapshot = Some(std::sync::Arc::clone(&mcp_snapshot));
    let mut desktop = build_desktop(&root_dir, git_keys);

    // Restore session state (layout, editor tabs, unfolded dirs, kiro tabs)
    if let Some(ref sess) = saved_session {
        session::restore_session(&mut desktop, sess);
        session::restore_tabs(&mut desktop, sess, &root_dir, &app_state.settings.editor_defaults);
        session::restore_kiro_tabs(
            &mut desktop,
            &sess.kiro_sessions,
            &root_dir,
            &mut app_state.kiro_registry,
        );
    }

    // Build status bar
    let status = build_status_bar(
        Box::new(AppCompleter::new(root_dir.clone())),
        app_state.settings.clock_interval,
        root_dir.clone(),
        &app_state.settings.status_keys,
    );

    // Build program
    let mut program = Program::new(Box::new(status), Box::new(desktop));

    // Run
    let color_mode = detect_color_mode();
    let mut backend = CrosstermBackend::new(color_mode);
    app_state.waker = Some(backend.waker());

    program.run(&mut backend, |ctx| {
        handle_command(ctx, &mut app_state);
    });

    // Shutdown all LSP servers gracefully
    app_state.lsp.shutdown_all();

    // Save session on quit
    if let Some(desktop) = program
        .desktop_mut()
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<kairn::layout_group::LayoutGroup>())
    {
        session::save_session(desktop, &root_dir, &app_state.kiro_registry);
    }

    // Clean up MCP socket
    if mcp_socket.is_ok() {
        let _ = std::fs::remove_file(&socket_path);
    }

    Ok(())
}
