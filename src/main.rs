//! kairn — TUI IDE entry point.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use clap::Parser;
use txv_core::program::Program;
use txv_core::run::Backend;
use txv_render::backend::CrosstermBackend;
use txv_widgets::tiled_workspace::TiledWorkspace;

use kairn::build_desktop::build_workspace;
use kairn::commands::{OpenFileRequest, RootsChangedData, CM_OPEN_FILE, CM_OPEN_FILE_FOCUS, CM_ROOTS_CHANGED};
use kairn::completer::AppCompleter;
use kairn::config::load_config;
use kairn::handler::{handle_command, AppState};
use kairn::init;
use kairn::mcp::bridge::run_mcp_bridge;
use kairn::mcp::commands::McpCommandQueue;
use kairn::mcp::listener::SharedCommandQueue;
use kairn::mcp::socket_path::socket_path;
use kairn::session;
use kairn::startup;
use kairn::status::build_status_bar;
use kairn::views::tree::FileTreeView;
use txv_widgets::sidekick_manager::SidekickManager;

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
    startup::install_panic_hook();
    let cli = Cli::parse();

    if let Some(result) = handle_early_exit(&cli) {
        return result;
    }

    let root_dir = fs::canonicalize(&cli.path)?;
    let (root_dir, open_file) = startup::resolve_root_and_file(root_dir);
    let sock_path = socket_path(&root_dir);
    startup::check_already_running(&sock_path);

    let (mcp_snapshot, mcp_cmd_queue, mcp_socket) = startup::start_mcp(&root_dir, &sock_path);
    startup::init_logging(&cli.log_file, &cli.log_level)?;

    let settings = load_config(&root_dir);
    let saved_session = session::load_session(&root_dir);
    let git_keys = settings.git_keys().clone();
    let mut app_state = AppState::with_settings(root_dir.to_path_buf(), settings);
    app_state.set_mcp_snapshot(Arc::clone(&mcp_snapshot));
    let _ = startup::PANIC_MESSAGES.set(app_state.messages().clone());
    startup::configure_app_state(&mut app_state, &root_dir);

    let mut desktop = build_workspace(&root_dir, git_keys);
    desktop.set_wide_threshold(app_state.settings().layout_wide_threshold());
    // Apply tree icon setting
    if app_state.settings().tree_icons() {
        if let Some(panel) = desktop.panel_mut(0) {
            if let Some(view) = panel.view_at_mut(0) {
                if let Some(tree) = view.as_any_mut().and_then(|a| a.downcast_mut::<FileTreeView>()) {
                    tree.set_show_icons(true);
                }
            }
        }
    }
    if let Some(ref sess) = saved_session {
        startup::restore_saved_session(&mut desktop, sess, &root_dir, &mut app_state);
    }

    run_app(
        &root_dir,
        &mut app_state,
        desktop,
        &open_file,
        &saved_session,
        mcp_cmd_queue,
    )?;
    shutdown(&mut app_state, &root_dir, &sock_path, &mcp_socket);
    Ok(())
}

fn handle_early_exit(cli: &Cli) -> Option<anyhow::Result<()>> {
    if cli.mcp_connect {
        return Some(run_mcp_bridge().map_err(|e| anyhow::anyhow!("MCP bridge failed: {e}")));
    }
    if cli.init_home {
        return Some(init::init_home_config());
    }
    if cli.init_wp {
        return Some(init::init_wp_config(&cli.path));
    }
    if env::var("KAIRN_SUSPENDED").is_ok() {
        eprintln!("kairn is already running (suspended). Use 'exit' to return.");
        process::exit(1);
    }
    None
}

fn run_app(
    root_dir: &std::path::Path,
    app_state: &mut AppState,
    desktop: TiledWorkspace,
    open_file: &Option<PathBuf>,
    saved_session: &Option<session::schema::SessionState>,
    mcp_cmd_queue: SharedCommandQueue,
) -> anyhow::Result<()> {
    let mut completer = AppCompleter::new(root_dir.to_path_buf(), app_state.command_list().clone());
    completer.set_lsp_languages(app_state.lsp_languages().clone());
    completer.set_roots(app_state.completer_roots().clone());
    let status = build_status_bar(
        &desktop,
        Box::new(completer),
        app_state.settings().clock_interval(),
        root_dir.to_path_buf(),
        app_state.settings().status_keys(),
    );
    let mut program = Program::new(Box::new(status), Box::new(desktop));
    program.insert_named("sidekick", Box::new(SidekickManager::new()));
    let mut backend = init_backend(app_state, &mcp_cmd_queue);

    push_initial_open(&program, open_file, saved_session, root_dir);

    // Notify tree of restored roots
    if app_state.roots().paths().len() > 1 {
        let data = RootsChangedData::from_roots(app_state.roots());
        program.sink().push_broadcast(CM_ROOTS_CHANGED, Some(Box::new(data)));
    }

    program.run(&mut backend, |ctx| {
        handle_command(ctx, app_state);
    });

    app_state.lsp_shutdown_all();
    save_session_on_exit(&mut program, app_state, root_dir);
    Ok(())
}

fn save_session_on_exit(program: &mut Program, app_state: &AppState, root_dir: &std::path::Path) {
    if let Some(desktop) = program
        .desktop_mut()
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<TiledWorkspace>())
    {
        let roots = app_state.roots().paths();
        if let Err(e) = session::save_session(desktop, root_dir, app_state.kiro_registry(), &roots) {
            log::warn!("session save: {e}");
        }
    }
}

fn init_backend(app_state: &mut AppState, mcp_cmd_queue: &SharedCommandQueue) -> CrosstermBackend {
    let color_mode = startup::detect_truecolor_mode();
    let backend = CrosstermBackend::new(color_mode);
    app_state.set_waker(backend.waker());
    app_state.lsp_set_waker(backend.waker());

    let cmd_queue = McpCommandQueue::new(backend.waker());
    app_state.set_mcp_commands(cmd_queue.clone());
    if let Ok(mut guard) = mcp_cmd_queue.lock() {
        *guard = Some(cmd_queue);
    }
    backend
}

fn shutdown(
    _app_state: &mut AppState,
    _root_dir: &std::path::Path,
    sock_path: &std::path::Path,
    mcp_socket: &Result<PathBuf, String>,
) {
    if mcp_socket.is_ok() {
        let _ = fs::remove_file(sock_path);
    }
}

fn push_initial_open(
    program: &Program,
    open_file: &Option<PathBuf>,
    saved_session: &Option<session::schema::SessionState>,
    root_dir: &std::path::Path,
) {
    if let Some(ref file_path) = open_file {
        program.sink().push_command(
            CM_OPEN_FILE_FOCUS,
            Some(Box::new(OpenFileRequest::new(file_path.clone()))),
        );
    } else if let Some(ref sess) = saved_session {
        for tab in sess.editor_tabs() {
            let path = root_dir.join(tab.path());
            program
                .sink()
                .push_command(CM_OPEN_FILE, Some(Box::new(OpenFileRequest::new(path))));
        }
    }
}
