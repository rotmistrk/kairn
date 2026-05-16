//! Configuration file loading from ~/.config/kairn/init.tcl via rusticle.

use std::path::Path;

use rusticle::interpreter::Interpreter;

use crate::config_keys::parse_key_var;
use crate::settings::AppSettings;

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
    if let Some(val) = interp.get_var("lsp.timeout") {
        if let Ok(n) = val.as_int() {
            settings.lsp_timeout = (n as u64).max(1);
        }
    }
    if let Some(val) = interp.get_var("git.stage") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.git_keys.stage = k;
        }
    }
    if let Some(val) = interp.get_var("git.unstage") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.git_keys.unstage = k;
        }
    }
    if let Some(val) = interp.get_var("git.untrack") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.git_keys.untrack = k;
        }
    }
    if let Some(val) = interp.get_var("git.commit") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.git_keys.commit = k;
        }
    }

    // Status bar key bindings
    if let Some(val) = interp.get_var("keys.help") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.help = k;
        }
    }
    if let Some(val) = interp.get_var("keys.tree") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.tree = k;
        }
    }
    if let Some(val) = interp.get_var("keys.main") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.main = k;
        }
    }
    if let Some(val) = interp.get_var("keys.term") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.term = k;
        }
    }
    if let Some(val) = interp.get_var("keys.zoom") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.zoom = k;
        }
    }
    if let Some(val) = interp.get_var("keys.messages") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.messages = k;
        }
    }
    if let Some(val) = interp.get_var("keys.quit") {
        if let Some(k) = parse_key_var(&val.as_str()) {
            settings.status_keys.quit = k;
        }
    }

    settings
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
