//! Startup helpers — logging, panic hook, color detection, session restore.

use std::env;
use std::fs::File;
use std::os::unix::net::UnixStream;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use crossterm::terminal::disable_raw_mode;
use env_logger::{Builder as LogBuilder, Target as LogTarget};
use log::LevelFilter;
use txv_core::glyphs::{detect_glyph_tier, set_glyphs, GlyphSet, GlyphTier};
use txv_core::message::Message;
use txv_core::palette::{self as pal, set_palette, ThemeMode};
use txv_render::color::{detect_color_mode, ColorMode};
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::completer::refresh_commands;
use crate::config_colors::apply_chrome_config;
use crate::desktop::SlotId;
use crate::handler::AppState;
use crate::handler_exec_table2::refresh_completer_roots;
use crate::handler_lsp_cmd::refresh_lsp_languages;
use crate::mcp::agent_file::write_agent_file;
use crate::mcp::listener::{start_mcp_listener, SharedCommandQueue};
use crate::mcp::snapshot::McpSnapshot;
use crate::message_ring::MessageRing;
use crate::project_root::detect_project_root;
use crate::session;
use crate::theme_state::ThemeState;

/// Global message ring — allows panic hook to report to the user.
pub static PANIC_MESSAGES: OnceLock<Arc<Mutex<MessageRing>>> = OnceLock::new();

pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let is_main = thread::current().name() == Some("main");
        if is_main {
            let _ = disable_raw_mode();
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
            log::error!("panic on thread {:?}: {info}", thread::current().name());
            if let Some(ring) = PANIC_MESSAGES.get() {
                if let Ok(mut r) = ring.lock() {
                    use txv_core::message::Message;
                    let msg = format!(
                        "Background thread crashed: {:?}. File watching may be degraded.",
                        thread::current().name()
                    );
                    r.push(Message::error("panic", msg));
                }
            }
        }
    }));
}

pub fn resolve_root_and_file(root_dir: PathBuf) -> (PathBuf, Option<PathBuf>) {
    if root_dir.is_file() {
        let home = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root_dir.parent().unwrap_or(&root_dir).to_path_buf());
        let project = detect_project_root(&root_dir, &home);
        (project, Some(root_dir))
    } else {
        (root_dir, None)
    }
}

pub fn init_logging(log_file_path: &Path, log_level: &str) -> anyhow::Result<()> {
    let log_file = File::create(log_file_path)?;
    LogBuilder::new()
        .filter_level(log_level.parse().unwrap_or(LevelFilter::Info))
        .target(LogTarget::Pipe(Box::new(log_file)))
        .init();
    Ok(())
}

pub fn configure_app_state(app_state: &mut AppState, root_dir: &Path) {
    let config_warnings = app_state.script_mut().load_config(root_dir);
    app_state.add_plugin_dir(root_dir.join(".kairn/plugins"));
    let plugin_warnings = app_state.refresh_plugins();

    // Surface config/plugin errors to message ring
    for w in config_warnings.iter().chain(plugin_warnings.iter()) {
        log::warn!("{w}");
        if let Ok(mut ring) = app_state.messages.lock() {
            ring.push(Message::error("config", w.clone()));
        }
    }
    if !config_warnings.is_empty() || !plugin_warnings.is_empty() {
        app_state.show_messages_on_start = true;
    }
    refresh_commands(app_state.command_list(), app_state.script());
    refresh_lsp_languages(app_state);
    refresh_completer_roots(app_state);

    let theme_mode = match app_state.settings().theme_mode() {
        "dark" => ThemeMode::Dark,
        "light" => ThemeMode::Light,
        _ => ThemeMode::Auto,
    };
    let theme = ThemeState::new(theme_mode);
    theme.apply();
    let framework_pal = pal::palette();
    let custom_pal = apply_chrome_config(app_state.script().interpreter(), framework_pal);
    set_palette(custom_pal);
    app_state.set_theme_state(theme);

    let glyph_tier = match app_state.settings().theme_glyphs() {
        "ascii" => GlyphTier::Ascii,
        "utf" => GlyphTier::Unicode,
        "nerd" => GlyphTier::Nerd,
        _ => detect_glyph_tier(),
    };
    set_glyphs(GlyphSet::from_tier(glyph_tier));
}

pub fn restore_saved_session(
    desktop: &mut TiledWorkspace,
    sess: &session::schema::SessionState,
    root_dir: &Path,
    app_state: &mut AppState,
) {
    // Restore additional workspace roots from session.
    if !sess.roots().is_empty() {
        for root_str in sess.roots() {
            let path = PathBuf::from(root_str);
            if path.is_dir() {
                app_state.roots_mut().add(path);
            }
        }
    }
    session::restore_session(desktop, sess);
    session::restore_tabs(
        desktop,
        sess,
        root_dir,
        app_state.settings().editor_defaults(),
        app_state.current_syntax_theme(),
    );
    for tab in sess.editor_tabs() {
        let p = Path::new(tab.path());
        if p.is_absolute() && p.is_file() {
            app_state.broker_open(tab.path(), SlotId::Center, 0);
        }
    }
    patch_editor_clipboard(desktop, app_state);
    let kiro_settings = app_state.settings().kiro().clone();
    session::restore_kiro_tabs(
        desktop,
        sess.kiro_sessions(),
        root_dir,
        app_state.kiro_registry_mut(),
        &kiro_settings,
    );
}

/// Set clipboard + register handles on all editors restored from session.
fn patch_editor_clipboard(desktop: &mut TiledWorkspace, state: &AppState) {
    use crate::views::editor::EditorView;
    for slot in 0..4 {
        let Some(panel) = desktop.panel_mut(slot) else {
            continue;
        };
        for i in 0..panel.tab_count() {
            let Some(view) = panel.view_at_mut(i) else {
                continue;
            };
            let Some(any) = view.as_any_mut() else {
                continue;
            };
            if let Some(ev) = any.downcast_mut::<EditorView>() {
                ev.editor_mut()
                    .set_shared_state(state.shared_register.clone(), state.clipboard.clone());
            }
        }
    }
}

pub fn start_mcp(
    root_dir: &Path,
    sock_path: &Path,
) -> (Arc<Mutex<McpSnapshot>>, SharedCommandQueue, Result<PathBuf, String>) {
    let mcp_snapshot = Arc::new(Mutex::new(McpSnapshot::default()));
    let mcp_cmd_queue: SharedCommandQueue = Arc::new(Mutex::new(None));
    let mcp_socket = start_mcp_listener(Arc::clone(&mcp_snapshot), Arc::clone(&mcp_cmd_queue), sock_path);
    if let Ok(ref sock) = mcp_socket {
        write_agent_file(root_dir);
        env::set_var("KAIRN_MCP_SOCKET", sock.to_string_lossy().as_ref());
    }
    (mcp_snapshot, mcp_cmd_queue, mcp_socket)
}

pub fn check_already_running(sock_path: &Path) {
    if UnixStream::connect(sock_path).is_ok() {
        eprintln!("kairn is already running for this project.");
        process::exit(1);
    }
}

/// Enhanced color mode detection — handles tmux truecolor support.
pub fn detect_truecolor_mode() -> ColorMode {
    let base = detect_color_mode();
    if base == ColorMode::TrueColor {
        return base;
    }
    let Ok(term) = env::var("TERM") else {
        return base;
    };
    if !term.starts_with("tmux") && !term.starts_with("screen") {
        return base;
    }
    let Ok(out) = Command::new("tmux")
        .args(["display", "-p", "#{client_termfeatures}"])
        .output()
    else {
        return base;
    };
    let features = String::from_utf8_lossy(&out.stdout);
    if features.contains("RGB") {
        ColorMode::TrueColor
    } else {
        base
    }
}
