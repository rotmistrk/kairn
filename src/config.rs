//! Configuration file loading from ~/.config/kairn/init.tcl via rusticle.

use std::path::Path;

use rusticle::interpreter::Interpreter;
use txv_core::prelude::*;

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

/// Parse a simple key variable value into a KeyEvent.
/// Supports: single chars ("s"), modifier combos ("Ctrl-s"), function keys ("F1").
fn parse_key_var(spec: &str) -> Option<KeyEvent> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    let parts: Vec<&str> = spec.split('-').collect();
    let mut modifiers = KeyMod::default();
    let key_part = parts.last()?;
    for &part in &parts[..parts.len().saturating_sub(1)] {
        match part {
            "Ctrl" | "ctrl" => modifiers.ctrl = true,
            "Alt" | "alt" => modifiers.alt = true,
            "Shift" | "shift" => modifiers.shift = true,
            _ => {}
        }
    }
    let code = parse_key_code(key_part)?;
    Some(KeyEvent { code, modifiers })
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    if s.len() == 1 {
        return Some(KeyCode::Char(s.chars().next()?));
    }
    // Function keys: F1..F12
    if let Some(n) = s.strip_prefix('F').or_else(|| s.strip_prefix('f')) {
        if let Ok(num) = n.parse::<u8>() {
            return Some(KeyCode::F(num));
        }
    }
    match s {
        "Esc" | "esc" => Some(KeyCode::Esc),
        "Enter" | "enter" => Some(KeyCode::Enter),
        "Tab" | "tab" => Some(KeyCode::Tab),
        "Left" | "left" => Some(KeyCode::Left),
        "Right" | "right" => Some(KeyCode::Right),
        "Up" | "up" => Some(KeyCode::Up),
        "Down" | "down" => Some(KeyCode::Down),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn write_config(dir: &Path, content: &str) -> PathBuf {
        let file = dir.join("init.tcl");
        fs::write(&file, content).unwrap();
        file
    }

    #[test]
    fn default_settings_when_no_config_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.tcl");
        let s = load_config_from(&path);
        assert!(s.editor_defaults.wrap);
        assert!(!s.editor_defaults.list);
        assert_eq!(s.editor_defaults.tabstop, 4);
        assert!(s.editor_defaults.number);
        assert_eq!(s.clock_interval, 60);
    }

    #[test]
    fn config_sets_editor_wrap_off() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_config(tmp.path(), "set editor.wrap off");
        let s = load_config_from(&path);
        assert!(!s.editor_defaults.wrap);
    }

    #[test]
    fn config_sets_tabstop() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_config(tmp.path(), "set editor.tabstop 8");
        let s = load_config_from(&path);
        assert_eq!(s.editor_defaults.tabstop, 8);
    }

    #[test]
    fn config_sets_clock_interval() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_config(tmp.path(), "set clock.interval 30");
        let s = load_config_from(&path);
        assert_eq!(s.clock_interval, 30);
    }

    #[test]
    fn config_ignores_unknown_variables() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_config(tmp.path(), "set unknown.thing foo");
        let s = load_config_from(&path);
        // Should return defaults without panic
        assert_eq!(s.clock_interval, 60);
        assert!(s.editor_defaults.wrap);
    }

    #[test]
    fn config_handles_syntax_error_gracefully() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_config(tmp.path(), "{{{");
        let s = load_config_from(&path);
        // Should return defaults without panic
        assert_eq!(s.clock_interval, 60);
        assert!(s.editor_defaults.wrap);
    }

    #[test]
    fn config_multiple_settings() {
        let tmp = tempfile::tempdir().unwrap();
        let script = "set editor.wrap off\n\
                      set editor.list on\n\
                      set editor.tabstop 2\n\
                      set editor.number off\n\
                      set clock.interval 120";
        let path = write_config(tmp.path(), script);
        let s = load_config_from(&path);
        assert!(!s.editor_defaults.wrap);
        assert!(s.editor_defaults.list);
        assert_eq!(s.editor_defaults.tabstop, 2);
        assert!(!s.editor_defaults.number);
        assert_eq!(s.clock_interval, 120);
    }
}
