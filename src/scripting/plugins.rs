//! Plugin hot-reload — scans plugin directories, loads/reloads/unloads plugins.
//!
//! Plugins are Tcl files in `~/.kairn/plugins/*/init.tcl`.
//! Each plugin's procs are tracked. On file change, old procs are removed
//! and the file is re-evaluated. On file deletion, procs are removed.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// State of a loaded plugin.
#[derive(Debug)]
struct PluginEntry {
    path: PathBuf,
    mtime: SystemTime,
    /// Proc names defined by this plugin.
    procs: Vec<String>,
}

/// Manages plugin lifecycle: load, reload, unload.
pub struct PluginManager {
    plugins: HashMap<String, PluginEntry>,
    plugin_dirs: Vec<PathBuf>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let mut plugin_dirs = Vec::new();
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(home).join(".kairn/plugins");
            if dir.is_dir() {
                plugin_dirs.push(dir);
            }
        }
        Self {
            plugins: HashMap::new(),
            plugin_dirs,
        }
    }

    /// Add a project-local plugin directory.
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        if dir.is_dir() && !self.plugin_dirs.contains(&dir) {
            self.plugin_dirs.push(dir);
        }
    }

    /// Scan plugin directories and apply changes. Returns list of warnings.
    pub fn refresh(&mut self, engine: &mut super::ScriptEngine) -> Vec<String> {
        let mut warnings = Vec::new();
        let discovered = self.discover_plugins();

        // Detect removed plugins
        let current_names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in &current_names {
            if !discovered.contains_key(name) {
                self.unload_plugin(name, engine);
            }
        }

        // Detect new or modified plugins
        for (name, path) in &discovered {
            let mtime = file_mtime(path);
            if let Some(existing) = self.plugins.get(name) {
                if existing.mtime == mtime {
                    continue; // unchanged
                }
                // Modified — reload
                let old_procs = existing.procs.clone();
                remove_procs(engine, &old_procs);
                match self.load_plugin_file(name, path, mtime, engine) {
                    Ok(()) => log::info!("plugin reloaded: {name}"),
                    Err(e) => {
                        warnings.push(format!("plugin {name}: {e}"));
                        log::warn!("plugin {name} reload failed: {e}");
                    }
                }
            } else {
                // New plugin
                match self.load_plugin_file(name, path, mtime, engine) {
                    Ok(()) => log::info!("plugin loaded: {name}"),
                    Err(e) => {
                        warnings.push(format!("plugin {name}: {e}"));
                        log::warn!("plugin {name} load failed: {e}");
                    }
                }
            }
        }

        // Check for conflicts (same proc in multiple plugins, or shadowing built-ins)
        let mut proc_owners: HashMap<&str, &str> = HashMap::new();
        for (name, entry) in &self.plugins {
            for proc_name in &entry.procs {
                if crate::completer::BUILTIN_COMMANDS.contains(&proc_name.as_str()) {
                    warnings.push(format!(
                        "plugin '{name}': proc '{proc_name}' shadows built-in command (will be ignored)"
                    ));
                } else if let Some(other) = proc_owners.get(proc_name.as_str()) {
                    warnings.push(format!(
                        "conflict: proc '{proc_name}' defined in both '{other}' and '{name}'"
                    ));
                } else {
                    proc_owners.insert(proc_name, name);
                }
            }
        }

        warnings
    }

    /// Discover all plugin init.tcl files across plugin directories.
    fn discover_plugins(&self) -> HashMap<String, PathBuf> {
        let mut found = HashMap::new();
        for dir in &self.plugin_dirs {
            let Ok(entries) = std::fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let init = entry.path().join("init.tcl");
                if init.is_file() {
                    found.insert(name, init);
                }
            }
        }
        found
    }

    fn unload_plugin(&mut self, name: &str, engine: &mut super::ScriptEngine) {
        if let Some(entry) = self.plugins.remove(name) {
            remove_procs(engine, &entry.procs);
            log::info!("plugin unloaded: {name}");
        }
    }

    fn load_plugin_file(
        &mut self,
        name: &str,
        path: &Path,
        mtime: SystemTime,
        engine: &mut super::ScriptEngine,
    ) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

        // Syntax check: validate before executing
        if let Err(e) = engine.validate(&content) {
            return Err(format!("syntax error: {e}"));
        }

        // Snapshot procs before eval
        let before: std::collections::HashSet<String> = engine.proc_names().into_iter().collect();

        // Evaluate
        engine.eval(&content).map_err(|e| format!("eval error: {e}"))?;

        // Determine which procs were added
        let after: std::collections::HashSet<String> = engine.proc_names().into_iter().collect();
        let new_procs: Vec<String> = after.difference(&before).cloned().collect();

        self.plugins.insert(
            name.to_string(),
            PluginEntry {
                path: path.to_path_buf(),
                mtime,
                procs: new_procs,
            },
        );
        Ok(())
    }
}

fn remove_procs(engine: &mut super::ScriptEngine, procs: &[String]) {
    for name in procs {
        engine.remove_proc(name);
    }
}

fn file_mtime(path: &Path) -> SystemTime {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
