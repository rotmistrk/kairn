//! Configuration loaded from .kairnrc search path.
//! Search order: $PWD/.kairnrc → $HOME/.kairnrc → built-in defaults.
//! Missing keys use defaults (sparse overlay). Collisions detected at startup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A key combination as stored in config.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo(pub String);

/// Where a keybinding came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    Default,
    Global,
    Project,
}

impl KeySource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Global => "~/.kairnrc",
            Self::Project => ".kairnrc",
        }
    }
}

/// Full configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_kiro_command")]
    pub kiro_command: String,
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    #[serde(default)]
    pub keys: HashMap<String, KeyCombo>,
    /// Source of each keybinding (not serialized).
    #[serde(skip)]
    pub key_sources: HashMap<String, KeySource>,
}

fn default_kiro_command() -> String {
    "kiro-cli".to_string()
}
fn default_true() -> bool {
    true
}
fn default_tab_width() -> usize {
    4
}

impl Config {
    /// Shell to use for new tabs — always $SHELL, falling back to /bin/sh.
    pub fn shell(&self) -> String {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

impl Default for Config {
    fn default() -> Self {
        let keys = default_keys();
        let key_sources = keys
            .keys()
            .map(|k| (k.clone(), KeySource::Default))
            .collect();
        Self {
            kiro_command: default_kiro_command(),
            line_numbers: true,
            tab_width: 4,
            keys,
            key_sources,
        }
    }
}

/// Built-in default keybindings.
pub fn default_keys() -> HashMap<String, KeyCombo> {
    let pairs: Vec<(&str, &str)> = [default_nav_keys(), default_action_keys()].concat();
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), KeyCombo(v.to_string())))
        .collect()
}

fn default_nav_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("quit", "ctrl+q"),
        ("rotate_layout", "ctrl+l"),
        ("toggle_tree", "ctrl+b"),
        ("cycle_focus", "ctrl+tab"),
        ("focus_tree", "f3"),
        ("focus_main", "f4"),
        ("focus_terminal", "f5"),
        ("peek_screen", "ctrl+o"),
        ("launch_editor", "ctrl+e"),
        ("suspend_to_shell", "ctrl+t"),
        ("open_search", "ctrl+p"),
        ("diff_current_file", "ctrl+d"),
        ("git_log", "ctrl+g"),
        ("show_help", "f1"),
        ("save_session", "ctrl+x s"),
        ("load_session", "ctrl+shift+o"),
    ]
}

fn default_action_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("new_kiro_tab", "ctrl+x n"),
        ("new_shell_tab", "ctrl+x t"),
        ("close_tab", "ctrl+x k"),
        ("capture_output", "ctrl+x o"),
        ("capture_all", "ctrl+x a"),
        ("save_buffer", "ctrl+x ctrl+s"),
        ("prev_tab", "alt+left"),
        ("next_tab", "alt+right"),
        ("resize_tree_shrink", "f7"),
        ("resize_tree_grow", "f8"),
        ("resize_interactive_grow", "f9"),
        ("resize_interactive_shrink", "f10"),
        ("resize_tree_shrink5", "shift+f7"),
        ("resize_tree_grow5", "shift+f8"),
        ("resize_interactive_grow5", "shift+f9"),
        ("resize_interactive_shrink5", "shift+f10"),
        ("scroll_up", "ctrl+up"),
        ("scroll_down", "ctrl+down"),
        ("scroll_top", "ctrl+home"),
        ("scroll_bottom", "ctrl+end"),
        ("cycle_mode_next", "ctrl+shift+down"),
        ("cycle_mode_prev", "ctrl+shift+up"),
        ("toggle_left_panel", "f6"),
        ("refresh_tree", "f11"),
        ("redraw", "f12"),
    ]
}

impl Config {
    /// Load config: merge $PWD/.kairnrc over $HOME/.kairnrc over defaults.
    /// Auto-creates $HOME/.kairnrc from defaults if absent.
    pub fn load(workspace: &Path) -> Self {
        Self::load_with_override(workspace, None)
    }

    pub fn load_with_override(workspace: &Path, explicit: Option<&Path>) -> Self {
        ensure_global_rc();
        let mut cfg = Self::default();
        if let Some(global) = load_rc_file(&global_rc_path()) {
            merge_into(&mut cfg, global, KeySource::Global);
        }
        let local_path = workspace.join(".kairnrc");
        if let Some(local) = load_rc_file(&local_path) {
            merge_into(&mut cfg, local, KeySource::Project);
        }
        // Explicit -C/--config overrides everything
        if let Some(path) = explicit {
            if let Some(over) = load_rc_file(path) {
                merge_into(&mut cfg, over, KeySource::Project);
            } else {
                eprintln!("kairn: cannot load config: {}", path.display());
                std::process::exit(78); // EX_CONFIG
            }
        }
        cfg
    }

    /// Detect keybinding collisions. Returns list of warnings.
    pub fn detect_collisions(&self) -> Vec<String> {
        let mut seen: HashMap<&str, Vec<&str>> = HashMap::new();
        for (action, combo) in &self.keys {
            if combo.0.is_empty() {
                continue;
            }
            seen.entry(combo.0.as_str())
                .or_default()
                .push(action.as_str());
        }
        seen.into_iter()
            .filter(|(_, actions)| actions.len() > 1)
            .map(|(key, actions)| {
                format!(
                    "⚠ Key conflict: {} is bound to: {}",
                    key,
                    actions.join(", ")
                )
            })
            .collect()
    }

    /// Format a binding for display: "action_name" → "Ctrl+Q"
    pub fn display_key(&self, action: &str) -> String {
        self.keys
            .get(action)
            .map(|k| k.0.clone())
            .unwrap_or_else(|| "unbound".to_string())
    }

    pub fn key_source(&self, action: &str) -> KeySource {
        self.key_sources
            .get(action)
            .copied()
            .unwrap_or(KeySource::Default)
    }

    /// Path to the global rc file.
    pub fn global_rc() -> PathBuf {
        global_rc_path()
    }
}

fn global_rc_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".kairnrc")
}

/// Create ~/.kairnrc from defaults if it doesn't exist.
fn ensure_global_rc() {
    let path = global_rc_path();
    if path.exists() {
        return;
    }
    let cfg = Config::default();
    if let Ok(json) = serde_json::to_string_pretty(&cfg) {
        let _ = std::fs::write(path, json);
    }
}

fn load_rc_file(path: &Path) -> Option<Config> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Merge overrides into base. Only non-default fields from overlay are applied.
fn merge_into(base: &mut Config, overlay: Config, source: KeySource) {
    if overlay.kiro_command != default_kiro_command() {
        base.kiro_command = overlay.kiro_command;
    }
    for (action, combo) in overlay.keys {
        base.keys.insert(action.clone(), combo);
        base.key_sources.insert(action, source);
    }
}
