//! kairn — TUI IDE entry point.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use clap::Parser;
use txv_core::program::Program;
use txv_core::run::Backend;
use txv_render::backend::CrosstermBackend;
use txv_render::color::{detect_color_mode, ColorMode};

use kairn::build_desktop::build_workspace;
use kairn::completer::AppCompleter;
use kairn::config::load_config;
use kairn::handler::{handle_command, AppState};
use kairn::message_ring::MessageRing;
use kairn::session;
use kairn::status::build_status_bar;

/// Global message ring — allows panic hook to report to the user.
static PANIC_MESSAGES: OnceLock<Arc<Mutex<MessageRing>>> = OnceLock::new();

#[derive(Parser)]
#[command(name = "kairn", about = "TUI IDE")]
struct Cli {
    /// File or directory to open
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

    /// Write default user config to ~/.config/kairn/init.tcl
    #[arg(long = "init-home")]
    init_home: bool,

    /// Write default project config to .kairn/init.tcl
    #[arg(long = "init-wp", alias = "init-workplace")]
    init_wp: bool,
}

fn main() -> anyhow::Result<()> {
    // Layer 3: Global panic handler — restore terminal before crashing
    std::panic::set_hook(Box::new(|info| {
        let is_main = std::thread::current().name() == Some("main");
        if is_main {
            // Restore terminal state only on main thread
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = crossterm::execute!(
                std::io::stderr(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show
            );
            eprintln!("\n\x1b[1;31mkairn panicked!\x1b[0m");
            eprintln!("{info}");
            if let Some(loc) = info.location() {
                eprintln!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
            }
            eprintln!("\nPlease report this bug.");
        } else {
            // Background thread panic — report to user via Messages panel
            log::error!("panic on thread {:?}: {info}", std::thread::current().name());
            if let Some(ring) = PANIC_MESSAGES.get() {
                if let Ok(mut r) = ring.lock() {
                    use txv_core::message::Message;
                    let msg = format!(
                        "Background thread crashed: {:?}. File watching may be degraded.",
                        std::thread::current().name()
                    );
                    r.push(Message::error("panic", msg));
                }
            }
        }
    }));

    let cli = Cli::parse();

    // MCP bridge mode: proxy stdin↔socket and exit
    if cli.mcp_connect {
        return kairn::mcp::bridge::run_mcp_bridge().map_err(|e| anyhow::anyhow!("MCP bridge failed: {e}"));
    }

    // Init modes: write default configs and exit
    if cli.init_home {
        return kairn::init::init_home_config();
    }
    if cli.init_wp {
        return kairn::init::init_wp_config(&cli.path);
    }

    // Nesting guard: prevent running inside a suspended kairn session
    if std::env::var("KAIRN_SUSPENDED").is_ok() {
        eprintln!("kairn is already running (suspended). Use 'exit' to return.");
        std::process::exit(1);
    }

    let root_dir = std::fs::canonicalize(&cli.path)?;

    // If path is a file, detect project root and open the file
    let (root_dir, open_file) = if root_dir.is_file() {
        let home = std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| root_dir.parent().unwrap_or(&root_dir).to_path_buf());
        let project = kairn::project_root::detect_project_root(&root_dir, &home);
        (project, Some(root_dir))
    } else {
        (root_dir, None)
    };

    // Compute socket path and check instance lock
    let socket_path = kairn::mcp::socket_path::socket_path(&root_dir);
    if std::os::unix::net::UnixStream::connect(&socket_path).is_ok() {
        eprintln!("kairn is already running for this project.");
        std::process::exit(1);
    }

    // Start MCP listener
    let mcp_snapshot = std::sync::Arc::new(std::sync::Mutex::new(kairn::mcp::snapshot::McpSnapshot::default()));
    let mcp_cmd_queue: kairn::mcp::listener::SharedCommandQueue = std::sync::Arc::new(std::sync::Mutex::new(None));
    let mcp_socket = kairn::mcp::listener::start_mcp_listener(
        std::sync::Arc::clone(&mcp_snapshot),
        std::sync::Arc::clone(&mcp_cmd_queue),
        &socket_path,
    );
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
    let _ = PANIC_MESSAGES.set(app_state.messages.clone());
    // Load Tcl config files (plugins may define new commands)
    app_state.script.load_config(&root_dir);
    app_state.plugins.add_plugin_dir(root_dir.join(".kairn/plugins"));
    let plugin_warnings = app_state.plugins.refresh(&mut app_state.script);
    for w in &plugin_warnings {
        log::warn!("plugin: {w}");
    }
    kairn::completer::refresh_commands(&app_state.command_list, &app_state.script);
    kairn::handler_lsp_cmd::refresh_lsp_languages(&app_state);
    // Initialize theme
    let theme_mode = match app_state.settings.theme_mode.as_str() {
        "dark" => txv_core::palette::ThemeMode::Dark,
        "light" => txv_core::palette::ThemeMode::Light,
        _ => txv_core::palette::ThemeMode::Auto,
    };
    let theme = kairn::theme_state::ThemeState::new(theme_mode);
    theme.apply();
    // Apply chrome color overrides from Tcl config
    let framework_pal = txv_core::palette::palette();
    let custom_pal = kairn::config_colors::apply_chrome_config(app_state.script.interpreter(), framework_pal);
    txv_core::palette::set_palette(custom_pal);
    app_state.theme_state = Some(std::cell::RefCell::new(theme));

    // Initialize glyphs
    let glyph_tier = match app_state.settings.theme_glyphs.as_str() {
        "ascii" => txv_core::glyphs::GlyphTier::Ascii,
        "utf" => txv_core::glyphs::GlyphTier::Unicode,
        "nerd" => txv_core::glyphs::GlyphTier::Nerd,
        _ => txv_core::glyphs::detect_glyph_tier(),
    };
    txv_core::glyphs::set_glyphs(txv_core::glyphs::GlyphSet::from_tier(glyph_tier));
    let mut desktop = build_workspace(&root_dir, git_keys);
    desktop.set_wide_threshold(app_state.settings.layout_wide_threshold);

    // Restore session state (layout, editor tabs, unfolded dirs, kiro tabs)
    if let Some(ref sess) = saved_session {
        session::restore_session(&mut desktop, sess);
        session::restore_tabs(
            &mut desktop,
            sess,
            &root_dir,
            &app_state.settings.editor_defaults,
            app_state.current_syntax_theme(),
        );
        // Register restored tabs with broker
        for tab in &sess.editor_tabs {
            app_state.broker.open(&tab.path, kairn::desktop::SlotId::Center, 0);
        }
        session::restore_kiro_tabs(
            &mut desktop,
            &sess.kiro_sessions,
            &root_dir,
            &mut app_state.kiro_registry,
        );
    }

    // Build status bar
    let mut completer = AppCompleter::new(root_dir.clone(), app_state.command_list.clone());
    completer.set_lsp_languages(app_state.lsp_languages.clone());
    let status = build_status_bar(
        &desktop,
        Box::new(completer),
        app_state.settings.clock_interval,
        root_dir.clone(),
        &app_state.settings.status_keys,
    );

    // Build program
    let mut program = Program::new(Box::new(status), Box::new(desktop));

    // Run
    let color_mode = detect_truecolor_mode();
    let mut backend = CrosstermBackend::new(color_mode);
    app_state.waker = Some(backend.waker());
    app_state.lsp.set_waker(backend.waker());

    // Now that waker is available, set up MCP command queue for write operations
    let cmd_queue = kairn::mcp::commands::McpCommandQueue::new(backend.waker());
    app_state.mcp_commands = Some(cmd_queue.clone());
    if let Ok(mut guard) = mcp_cmd_queue.lock() {
        *guard = Some(cmd_queue);
    }

    // Open file from CLI argument (if path was a file)
    if let Some(ref file_path) = open_file {
        program.sink().push_command(
            kairn::commands::CM_OPEN_FILE_FOCUS,
            Some(Box::new(kairn::commands::OpenFileRequest::new(file_path.clone()))),
        );
    } else if let Some(ref sess) = saved_session {
        // Trigger LSP didOpen for session-restored files
        for tab in &sess.editor_tabs {
            let path = root_dir.join(&tab.path);
            program.sink().push_command(
                kairn::commands::CM_OPEN_FILE,
                Some(Box::new(kairn::commands::OpenFileRequest::new(path))),
            );
        }
    }

    program.run(&mut backend, |ctx| {
        handle_command(ctx, &mut app_state);
    });

    // Shutdown all LSP servers gracefully
    app_state.lsp.shutdown_all();

    // Save session on quit
    if let Some(desktop) = program
        .desktop_mut()
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<txv_widgets::tiled_workspace::TiledWorkspace>())
    {
        session::save_session(desktop, &root_dir, &app_state.kiro_registry);
    }

    // Clean up MCP socket
    if mcp_socket.is_ok() {
        let _ = std::fs::remove_file(&socket_path);
    }

    Ok(())
}

/// Enhanced color mode detection — handles tmux truecolor support.
/// tmux doesn't propagate COLORTERM but does advertise RGB in termfeatures.
fn detect_truecolor_mode() -> ColorMode {
    let base = detect_color_mode();
    if base == ColorMode::TrueColor {
        return base;
    }
    // tmux with RGB support
    if let Ok(term) = std::env::var("TERM") {
        if term.starts_with("tmux") || term.starts_with("screen") {
            if let Ok(out) = std::process::Command::new("tmux")
                .args(["display", "-p", "#{client_termfeatures}"])
                .output()
            {
                let features = String::from_utf8_lossy(&out.stdout);
                if features.contains("RGB") {
                    return ColorMode::TrueColor;
                }
            }
        }
    }
    base
}
