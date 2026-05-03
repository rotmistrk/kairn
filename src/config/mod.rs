//! Tcl-based configuration loading via rusticle.
//!
//! Config files are evaluated in order: embedded defaults → global
//! (`~/.kairnrc.tcl`) → project (`$PWD/.kairnrc.tcl`). They share
//! one interpreter instance so later files can override earlier ones.

pub mod keybindings;
pub mod themes;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use keybindings::{BindingSource, BindingTable, BoundAction, KeySpec};
use themes::ThemeValues;

/// Embedded default config script.
const DEFAULTS_TCL: &str = include_str!("defaults.tcl");

/// Embedded init template.
const INIT_TEMPLATE: &str = include_str!("init_template.tcl");

// ── Config types ─────────────────────────

/// Supported keymap modes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keymap {
    /// Vim modal editing.
    Vi,
    /// Emacs chord-based editing.
    Emacs,
    /// Classic menu-driven editing.
    Classic,
}

/// Typed config values extracted from the kairn context.
pub struct KairnConfig {
    /// Active keymap mode.
    pub keymap: Keymap,
    /// Tab width in spaces.
    pub tab_width: u16,
    /// Whether to show line numbers.
    pub line_numbers: bool,
    /// Whether to auto-save on focus loss.
    pub auto_save: bool,
    /// Active theme name.
    pub theme_name: String,
    /// Shell command (empty = $SHELL).
    pub shell: String,
    /// Kiro CLI command.
    pub kiro_command: String,
}

/// Result of config loading.
pub struct ConfigResult {
    /// The interpreter with all state loaded.
    pub interp: Interpreter,
    /// Extracted typed config values.
    pub config: KairnConfig,
    /// Keybinding table.
    pub bindings: BindingTable,
    /// Theme values.
    pub theme: ThemeValues,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Config loading errors.
#[derive(Debug)]
pub enum ConfigError {
    /// Script evaluation failed.
    Script { path: PathBuf, error: TclError },
    /// IO error reading config file.
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Script { path, error } => {
                write!(f, "{}: {error}", path.display())
            }
            Self::Io { path, error } => {
                write!(f, "{}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

// ── ConfigLoader ─────────────────────────

/// Loads config files in order and extracts typed values.
pub struct ConfigLoader {
    workspace: PathBuf,
    explicit_config: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a loader for the given workspace.
    pub fn new(workspace: &Path) -> Self {
        Self {
            workspace: workspace.to_path_buf(),
            explicit_config: None,
        }
    }

    /// Add an explicit config file to load after project config.
    pub fn with_explicit_config(mut self, path: PathBuf) -> Self {
        self.explicit_config = Some(path);
        self
    }

    /// Execute the full loading sequence.
    pub fn load(self) -> Result<ConfigResult, ConfigError> {
        let mut interp = Interpreter::new();
        let bindings = Arc::new(Mutex::new(BindingTable::new()));
        let theme = Arc::new(Mutex::new(ThemeValues::new()));
        let mut warnings = Vec::new();

        // Register bind and theme commands
        register_bind_cmd(&mut interp, &bindings, BindingSource::Default);
        register_theme_cmd(&mut interp, &theme);

        // 1. Evaluate embedded defaults
        eval_script(&mut interp, DEFAULTS_TCL, &PathBuf::from("<defaults>"))?;

        // 2. Global config
        let global = global_rc_path();
        if global.exists() {
            let source = BindingSource::GlobalConfig;
            update_bind_source(&mut interp, &bindings, source);
            eval_file(&mut interp, &global)?;
        }

        // 3. Project config
        let project = self.workspace.join(".kairnrc.tcl");
        if project.exists() {
            let source = BindingSource::ProjectConfig;
            update_bind_source(&mut interp, &bindings, source);
            eval_file(&mut interp, &project)?;
        }

        // 4. Explicit override
        if let Some(ref path) = self.explicit_config {
            let source = BindingSource::ProjectConfig;
            update_bind_source(&mut interp, &bindings, source);
            eval_file(&mut interp, path)?;
        }

        // Extract typed config
        let config = extract_config(&interp, &mut warnings);

        // Extract from Arc<Mutex<>>
        let bindings = Arc::try_unwrap(bindings)
            .map_err(|_| ConfigError::Script {
                path: PathBuf::from("<internal>"),
                error: TclError::new("binding table still shared"),
            })?
            .into_inner()
            .map_err(|_| ConfigError::Script {
                path: PathBuf::from("<internal>"),
                error: TclError::new("binding table lock poisoned"),
            })?;

        let mut theme = Arc::try_unwrap(theme)
            .map_err(|_| ConfigError::Script {
                path: PathBuf::from("<internal>"),
                error: TclError::new("theme still shared"),
            })?
            .into_inner()
            .map_err(|_| ConfigError::Script {
                path: PathBuf::from("<internal>"),
                error: TclError::new("theme lock poisoned"),
            })?;

        theme.apply_from_context(&interp);

        Ok(ConfigResult {
            interp,
            config,
            bindings,
            theme,
            warnings,
        })
    }

    /// Write the init template to `~/.kairnrc.tcl`.
    /// Returns the path written, or `None` if file already exists.
    pub fn init_config() -> Result<Option<PathBuf>, ConfigError> {
        let path = global_rc_path();
        init_config_at(&path)
    }
}

/// Write init template to a specific path (for testing).
pub fn init_config_at(path: &Path) -> Result<Option<PathBuf>, ConfigError> {
    if path.exists() {
        return Ok(None);
    }
    std::fs::write(path, INIT_TEMPLATE).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        error: e,
    })?;
    Ok(Some(path.to_path_buf()))
}

// ── Backward-compatible Config struct ────

/// Where a keybinding came from (backward compat).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    /// Built-in default.
    Default,
    /// From `~/.kairnrc` or `~/.kairnrc.tcl`.
    Global,
    /// From `$PWD/.kairnrc` or `$PWD/.kairnrc.tcl`.
    Project,
}

impl KeySource {
    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Global => "~/.kairnrc",
            Self::Project => ".kairnrc",
        }
    }
}

/// A key combination as stored in config (backward compat).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct KeyCombo(pub String);

/// Full configuration (backward-compatible API for app.rs).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Kiro CLI command.
    #[serde(default = "default_kiro_command")]
    pub kiro_command: String,
    /// Whether to show line numbers.
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    /// Tab width in spaces.
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    /// Keybindings: action name → key combo.
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
    /// Shell to use for new tabs.
    pub fn shell(&self) -> String {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }

    /// Load config from workspace (backward-compatible entry point).
    pub fn load(workspace: &Path) -> Self {
        Self::load_with_override(workspace, None)
    }

    /// Load config with optional explicit override file.
    pub fn load_with_override(workspace: &Path, explicit: Option<&Path>) -> Self {
        // Try new Tcl config first
        let tcl_global = global_rc_path();
        let tcl_project = workspace.join(".kairnrc.tcl");
        let has_tcl = tcl_global.exists() || tcl_project.exists();

        if has_tcl {
            return Self::from_tcl(workspace, explicit);
        }

        // Fall back to old JSON config
        Self::from_json(workspace, explicit)
    }

    /// Format a binding for display.
    pub fn display_key(&self, action: &str) -> String {
        self.keys
            .get(action)
            .map(|k| k.0.clone())
            .unwrap_or_else(|| "unbound".to_string())
    }

    /// Get the source of a keybinding.
    pub fn key_source(&self, action: &str) -> KeySource {
        self.key_sources
            .get(action)
            .copied()
            .unwrap_or(KeySource::Default)
    }

    /// Detect keybinding collisions.
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
            .map(|(key, actions)| format!("Key conflict: {} bound to: {}", key, actions.join(", ")))
            .collect()
    }

    /// Path to the global rc file.
    pub fn global_rc() -> PathBuf {
        global_rc_path()
    }

    fn from_tcl(workspace: &Path, explicit: Option<&Path>) -> Self {
        let mut loader = ConfigLoader::new(workspace);
        if let Some(p) = explicit {
            loader = loader.with_explicit_config(p.to_path_buf());
        }
        match loader.load() {
            Ok(result) => Self::from_config_result(result),
            Err(e) => {
                eprintln!("kairn: config error: {e}");
                std::process::exit(78);
            }
        }
    }

    fn from_config_result(result: ConfigResult) -> Self {
        let keys = build_compat_keys(&result.bindings);
        let key_sources = keys
            .keys()
            .map(|k| (k.clone(), KeySource::Default))
            .collect();
        Self {
            kiro_command: result.config.kiro_command,
            line_numbers: result.config.line_numbers,
            tab_width: result.config.tab_width as usize,
            keys,
            key_sources,
        }
    }

    fn from_json(workspace: &Path, explicit: Option<&Path>) -> Self {
        let mut cfg = Self::default();
        let global = json_rc_path();
        if let Some(g) = load_json_rc(&global) {
            merge_json(&mut cfg, g, KeySource::Global);
        }
        let local = workspace.join(".kairnrc");
        if let Some(l) = load_json_rc(&local) {
            merge_json(&mut cfg, l, KeySource::Project);
        }
        if let Some(path) = explicit {
            if let Some(o) = load_json_rc(path) {
                merge_json(&mut cfg, o, KeySource::Project);
            }
        }
        cfg
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

// ── Internal helpers ─────────────────────

fn global_rc_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".kairnrc.tcl")
}

fn json_rc_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".kairnrc")
}

fn eval_script(interp: &mut Interpreter, script: &str, path: &Path) -> Result<(), ConfigError> {
    interp.eval(script).map_err(|e| ConfigError::Script {
        path: path.to_path_buf(),
        error: e,
    })?;
    Ok(())
}

fn eval_file(interp: &mut Interpreter, path: &Path) -> Result<(), ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        error: e,
    })?;
    eval_script(interp, &content, path)
}

fn extract_config(interp: &Interpreter, warnings: &mut Vec<String>) -> KairnConfig {
    let keymap = match get_ctx_str(interp, "kairn::keymap").as_str() {
        "emacs" => Keymap::Emacs,
        "classic" => Keymap::Classic,
        "vi" => Keymap::Vi,
        other => {
            warnings.push(format!("unknown keymap: {other}, using vi"));
            Keymap::Vi
        }
    };
    KairnConfig {
        keymap,
        tab_width: get_ctx_int(interp, "kairn::tab-width", 4) as u16,
        line_numbers: get_ctx_bool(interp, "kairn::line-numbers", true),
        auto_save: get_ctx_bool(interp, "kairn::auto-save", false),
        theme_name: get_ctx_str(interp, "kairn::theme"),
        shell: get_ctx_str(interp, "kairn::shell"),
        kiro_command: get_ctx_str(interp, "kairn::kiro-command"),
    }
}

fn get_ctx_str(interp: &Interpreter, name: &str) -> String {
    interp
        .get_var(name)
        .map(|v| v.as_str().to_string())
        .unwrap_or_default()
}

fn get_ctx_int(interp: &Interpreter, name: &str, default: i64) -> i64 {
    interp
        .get_var(name)
        .and_then(|v| v.as_int().ok())
        .unwrap_or(default)
}

fn get_ctx_bool(interp: &Interpreter, name: &str, default: bool) -> bool {
    interp
        .get_var(name)
        .and_then(|v| v.as_bool().ok())
        .unwrap_or(default)
}

use std::sync::{Arc, Mutex};

/// Shared binding table for use in closures.
type SharedBindings = Arc<Mutex<BindingTable>>;

/// Shared theme values for use in closures.
type SharedTheme = Arc<Mutex<ThemeValues>>;

/// Register the `bind` command that populates the binding table.
fn register_bind_cmd(interp: &mut Interpreter, bindings: &SharedBindings, source: BindingSource) {
    let b = bindings.clone();
    let src = source;
    interp.register_fn("bind", move |_interp, args| {
        let keyspec_str = require_arg(args, 0, "bind")?;
        let script = require_arg(args, 1, "bind")?;
        let ks = KeySpec::parse(&keyspec_str).map_err(|e| TclError::new(format!("bind: {e}")))?;
        let mut table = b.lock().map_err(|_| TclError::new("lock poisoned"))?;
        table.bind(
            ks,
            BoundAction {
                script,
                source: src.clone(),
            },
        );
        Ok(TclValue::Str(String::new()))
    });
}

/// Update the binding source for subsequent bind calls.
fn update_bind_source(interp: &mut Interpreter, bindings: &SharedBindings, source: BindingSource) {
    register_bind_cmd(interp, bindings, source);
}

/// Register the `theme` command.
fn register_theme_cmd(interp: &mut Interpreter, theme: &SharedTheme) {
    let t = theme.clone();
    interp.register_fn("theme", move |interp, args| {
        let sub = require_arg(args, 0, "theme")?;
        let mut tv = t.lock().map_err(|_| TclError::new("lock poisoned"))?;
        match sub.as_str() {
            "load" => {
                let name = require_arg(args, 1, "theme load")?;
                let script = tv
                    .find_theme_script(&name)
                    .ok_or_else(|| TclError::new(format!("theme: unknown theme \"{name}\"")))?;
                drop(tv); // release lock before eval
                interp.eval(&script)?;
                let mut tv2 = t.lock().map_err(|_| TclError::new("lock poisoned"))?;
                tv2.apply_from_context(interp);
                Ok(TclValue::Str(String::new()))
            }
            "list" => {
                let names = tv.available_themes();
                Ok(TclValue::List(
                    names.into_iter().map(TclValue::Str).collect(),
                ))
            }
            "get" => {
                let prop = require_arg(args, 1, "theme get")?;
                let val = tv
                    .get(&prop)
                    .ok_or_else(|| TclError::new(format!("theme: unknown property \"{prop}\"")))?;
                Ok(TclValue::Str(val.to_string()))
            }
            "set" => {
                let prop = require_arg(args, 1, "theme set")?;
                let val = require_arg(args, 2, "theme set")?;
                tv.set(&prop, &val);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!(
                "theme: unknown subcommand \"{sub}\""
            ))),
        }
    });
}

fn require_arg(args: &[TclValue], index: usize, cmd: &str) -> Result<String, TclError> {
    args.get(index)
        .map(|v| v.as_str().into_owned())
        .ok_or_else(|| TclError::new(format!("{cmd}: missing argument {index}")))
}

// ── Backward-compat key mapping ──────────

fn build_compat_keys(bindings: &BindingTable) -> HashMap<String, KeyCombo> {
    let mut keys = HashMap::new();
    for (spec_str, action) in bindings.all_bindings() {
        // Map script to action name for backward compat
        let action_name = script_to_action_name(&action.script);
        keys.insert(action_name, KeyCombo(spec_str));
    }
    keys
}

fn script_to_action_name(script: &str) -> String {
    // "editor quit" → "quit", "buffer save" → "save_buffer"
    script.trim().replace([' ', '-'], "_")
}

// ── JSON backward compat ─────────────────

fn load_json_rc(path: &Path) -> Option<Config> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn merge_json(base: &mut Config, overlay: Config, source: KeySource) {
    if overlay.kiro_command != default_kiro_command() {
        base.kiro_command = overlay.kiro_command;
    }
    for (action, combo) in overlay.keys {
        base.keys.insert(action.clone(), combo);
        base.key_sources.insert(action, source);
    }
}

/// Built-in default keybindings (JSON format).
pub fn default_keys() -> HashMap<String, KeyCombo> {
    let pairs = vec![
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
    ];
    pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), KeyCombo(v.to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn embedded_defaults_parse() {
        let mut interp = Interpreter::new();
        let bindings = Arc::new(Mutex::new(BindingTable::new()));
        let theme = Arc::new(Mutex::new(ThemeValues::new()));
        register_bind_cmd(&mut interp, &bindings, BindingSource::Default);
        register_theme_cmd(&mut interp, &theme);
        let result = interp.eval(DEFAULTS_TCL);
        assert!(result.is_ok(), "defaults.tcl failed: {result:?}");
    }

    #[test]
    fn extract_default_config() {
        let mut interp = Interpreter::new();
        let bindings = Arc::new(Mutex::new(BindingTable::new()));
        let theme = Arc::new(Mutex::new(ThemeValues::new()));
        register_bind_cmd(&mut interp, &bindings, BindingSource::Default);
        register_theme_cmd(&mut interp, &theme);
        interp.eval(DEFAULTS_TCL).unwrap();
        let mut warnings = Vec::new();
        let cfg = extract_config(&interp, &mut warnings);
        assert_eq!(cfg.keymap, Keymap::Vi);
        assert_eq!(cfg.tab_width, 4);
        assert!(cfg.line_numbers);
        assert!(!cfg.auto_save);
    }

    #[test]
    fn init_config_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".kairnrc.tcl");
        let result = init_config_at(&path);
        assert!(result.is_ok());
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("kairn::keymap"));
    }

    #[test]
    fn init_config_does_not_overwrite() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".kairnrc.tcl");
        std::fs::write(&path, "# my config\n").unwrap();
        let result = init_config_at(&path).unwrap();
        assert!(result.is_none());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "# my config\n");
    }

    #[test]
    fn default_config_has_keys() {
        let cfg = Config::default();
        assert!(cfg.keys.contains_key("quit"));
        assert!(cfg.keys.contains_key("toggle_tree"));
    }

    #[test]
    fn config_display_key() {
        let cfg = Config::default();
        assert_eq!(cfg.display_key("quit"), "ctrl+q");
    }

    #[test]
    fn config_shell_fallback() {
        let cfg = Config::default();
        let shell = cfg.shell();
        assert!(!shell.is_empty());
    }
}
