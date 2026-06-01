//! Configuration file loading from ~/.config/kairn/init.tcl via rusticle.

use std::path::Path;

use rusticle::interpreter::Interpreter;
use txv_core::prelude::KeyEvent;

use crate::config_keys::parse_key_var;
use crate::settings::{AppSettings, CursorStyle};

/// Load configuration from `$XDG_CONFIG_HOME/kairn/init.tcl` (or `~/.config/kairn/init.tcl`).
/// Returns defaults silently if the file does not exist or on any error.
pub fn load_config(_root_dir: &Path) -> AppSettings {
    let config_path = match config_file_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };
    load_config_from(&config_path)
}

/// Load configuration from a specific file path.
/// Returns defaults if the file does not exist or on any error.
pub fn load_config_from(path: &Path) -> AppSettings {
    if !path.exists() {
        return AppSettings::default();
    }

    let script = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to read config {}: {}", path.display(), e);
            return AppSettings::default();
        }
    };

    let mut interp = Interpreter::new();
    crate::lsp::config_commands::register_lsp_commands(&mut interp);
    if let Err(e) = interp.eval(&script) {
        log::warn!("Config eval error in {}: {}", path.display(), e);
        return AppSettings::default();
    }

    extract_settings(&interp)
}

/// Determine the config file path from XDG or fallback.
fn config_file_path() -> Option<std::path::PathBuf> {
    let config_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(xdg)
    } else {
        let home = std::env::var("HOME").ok()?;
        std::path::PathBuf::from(home).join(".config")
    };
    Some(config_dir.join("kairn").join("init.tcl"))
}

/// Read Tcl variables set by the script and map them to AppSettings.
fn extract_settings(interp: &Interpreter) -> AppSettings {
    let mut settings = AppSettings::default();
    extract_editor_settings(interp, &mut settings);
    extract_app_settings(interp, &mut settings);
    extract_key_settings(interp, &mut settings);
    extract_kiro_settings(interp, &mut settings);
    settings
}

fn extract_editor_settings(interp: &Interpreter, settings: &mut AppSettings) {
    if let Some(val) = interp.get_var("editor.wrap") {
        if let Ok(b) = val.as_bool() {
            settings.editor_defaults.wrap = b;
        }
    }
    if let Some(val) = interp.get_var("editor.list") {
        if let Ok(b) = val.as_bool() {
            settings.editor_defaults.list = b;
        }
    }
    if let Some(val) = interp.get_var("editor.tabstop") {
        if let Ok(n) = val.as_int() {
            settings.editor_defaults.tabstop = n as u16;
        }
    }
    if let Some(val) = interp.get_var("editor.number") {
        if let Ok(b) = val.as_bool() {
            settings.editor_defaults.number = b;
        }
    }
    if let Some(val) = interp.get_var("editor.cursor_insert") {
        if let Some(s) = parse_cursor_style(&val.to_string()) {
            settings.editor_defaults.cursor_insert = s;
        }
    }
    if let Some(val) = interp.get_var("editor.cursor_normal") {
        if let Some(s) = parse_cursor_style(&val.to_string()) {
            settings.editor_defaults.cursor_normal = s;
        }
    }
    if let Some(val) = interp.get_var("editor.cursor_command") {
        if let Some(s) = parse_cursor_style(&val.to_string()) {
            settings.editor_defaults.cursor_command = s;
        }
    }
    if let Some(val) = interp.get_var("editor.rainbow") {
        if let Ok(b) = val.as_bool() {
            settings.editor_defaults.rainbow = b;
        }
    }
    if let Some(val) = interp.get_var("editor.guides") {
        if let Ok(b) = val.as_bool() {
            settings.editor_defaults.guides = b;
        }
    }
}

fn extract_app_settings(interp: &Interpreter, settings: &mut AppSettings) {
    if let Some(val) = interp.get_var("clock.interval") {
        if let Ok(n) = val.as_int() {
            settings.clock_interval = n as u16;
        }
    }
    if let Some(val) = interp.get_var("terminal.scrollback") {
        if let Ok(n) = val.as_int() {
            settings.scrollback_lines = n as u16;
        }
    }
    if let Some(val) = interp.get_var("terminal.idle-timeout") {
        if let Ok(n) = val.as_int() {
            settings.terminal_idle_timeout = n as u64;
        }
    }
    if let Some(val) = interp.get_var("terminal.auto-close-on-exit") {
        settings.terminal_auto_close = val.as_str() == "true" || val.as_str() == "1";
    }
    if let Some(val) = interp.get_var("layout.wide-threshold") {
        if let Ok(n) = val.as_int() {
            settings.layout_wide_threshold = n as u16;
        }
    }
    if let Some(val) = interp.get_var("layout.tall-threshold") {
        if let Ok(n) = val.as_int() {
            settings.layout_tall_threshold = n as u16;
        }
    }
    if let Some(val) = interp.get_var("tabs.max") {
        if let Ok(n) = val.as_int() {
            settings.max_tabs = n as u16;
        }
    }
    if let Some(val) = interp.get_var("lsp.timeout") {
        if let Ok(n) = val.as_int() {
            settings.lsp_timeout = (n as u64).max(1);
        }
    }
    extract_theme_settings(interp, settings);
}

fn extract_theme_settings(interp: &Interpreter, settings: &mut AppSettings) {
    if let Some(val) = interp.get_var("theme.mode") {
        let s = val.as_str();
        if s == "dark" || s == "light" || s == "auto" {
            settings.theme_mode = s.to_string();
        }
    }
    if let Some(val) = interp.get_var("theme.syntax_dark") {
        settings.theme_syntax_dark = val.as_str().to_string();
    }
    if let Some(val) = interp.get_var("theme.syntax_light") {
        settings.theme_syntax_light = val.as_str().to_string();
    }
    if let Some(val) = interp.get_var("theme.glyphs") {
        let s = val.as_str();
        if s == "ascii" || s == "utf" || s == "nerd" || s == "auto" {
            settings.theme_glyphs = s.to_string();
        }
    }
}

fn extract_kiro_settings(interp: &Interpreter, settings: &mut AppSettings) {
    if let Some(val) = interp.get_var("kiro.cmd") {
        if let Ok(items) = val.as_list() {
            let v: Vec<String> = items.iter().map(|i| i.as_str().to_string()).collect();
            if !v.is_empty() {
                settings.kiro.cmd = v;
            }
        }
    }
    if let Some(val) = interp.get_var("kiro.resume-first") {
        if let Ok(items) = val.as_list() {
            settings.kiro.resume_first = items.iter().map(|i| i.as_str().to_string()).collect();
        }
    }
    if let Some(val) = interp.get_var("kiro.resume-rest") {
        if let Ok(items) = val.as_list() {
            settings.kiro.resume_rest = items.iter().map(|i| i.as_str().to_string()).collect();
        }
    }
}

type KeyAccessor = fn(&mut AppSettings) -> &mut KeyEvent;

fn extract_key_settings(interp: &Interpreter, settings: &mut AppSettings) {
    let key_map: &[(&str, KeyAccessor)] = &[
        ("git.stage", |s| &mut s.git_keys.stage),
        ("git.unstage", |s| &mut s.git_keys.unstage),
        ("git.untrack", |s| &mut s.git_keys.untrack),
        ("git.commit", |s| &mut s.git_keys.commit),
        ("keys.help", |s| &mut s.status_keys.help),
        ("keys.tree", |s| &mut s.status_keys.tree),
        ("keys.main", |s| &mut s.status_keys.main),
        ("keys.term", |s| &mut s.status_keys.term),
        ("keys.zoom", |s| &mut s.status_keys.zoom),
        ("keys.messages", |s| &mut s.status_keys.messages),
        ("keys.quit", |s| &mut s.status_keys.quit),
        ("keys.subpanel_focus", |s| &mut s.status_keys.subpanel_focus),
        ("keys.subpanel_move", |s| &mut s.status_keys.subpanel_move),
        ("keys.subpanel_grow", |s| &mut s.status_keys.subpanel_grow),
        ("keys.subpanel_shrink", |s| &mut s.status_keys.subpanel_shrink),
    ];
    for (var, accessor) in key_map {
        if let Some(val) = interp.get_var(var) {
            if let Some(k) = parse_key_var(&val.as_str()) {
                *accessor(settings) = k;
            }
        }
    }
}

fn parse_cursor_style(s: &str) -> Option<CursorStyle> {
    match s.to_lowercase().as_str() {
        "bar" => Some(CursorStyle::Bar),
        "block" => Some(CursorStyle::Block),
        "underline" => Some(CursorStyle::Underline),
        "software" | "none" => Some(CursorStyle::Software),
        _ => None,
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
