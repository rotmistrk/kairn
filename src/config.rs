//! Configuration file loading from ~/.config/kairn/init.tcl via rusticle.

use std::path::Path;

use rusticle::interpreter::Interpreter;

use crate::settings::AppSettings;

/// Load configuration from `$XDG_CONFIG_HOME/kairn/init.tcl` (or `~/.config/kairn/init.tcl`).
/// Returns defaults silently if the file does not exist or on any error.
pub fn load_config(_root_dir: &Path) -> AppSettings {
    let config_path = match config_file_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };

    if !config_path.exists() {
        return AppSettings::default();
    }

    let script = match std::fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to read config {}: {}", config_path.display(), e);
            return AppSettings::default();
        }
    };

    let mut interp = Interpreter::new();
    if let Err(e) = interp.eval(&script) {
        log::warn!("Config eval error in {}: {}", config_path.display(), e);
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

    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_defaults_when_no_vars() {
        let interp = Interpreter::new();
        let s = extract_settings(&interp);
        assert!(s.editor_defaults.wrap);
        assert!(!s.editor_defaults.list);
        assert_eq!(s.editor_defaults.tabstop, 4);
        assert_eq!(s.clock_interval, 60);
    }

    #[test]
    fn test_extract_settings_from_script() {
        let mut interp = Interpreter::new();
        interp.eval("set editor.wrap off\nset clock.interval 30").ok();
        let s = extract_settings(&interp);
        assert!(!s.editor_defaults.wrap);
        assert_eq!(s.clock_interval, 30);
    }
}

