//! Plugin hot-reload — scans plugin directories, loads/reloads/unloads plugins.
//!
//! Plugins are Tcl files in `~/.kairn/plugins/*/init.tcl`.
//! Each plugin's procs are tracked. On file change, old procs are removed
//! and the file is re-evaluated. On file deletion, procs are removed.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::completer::BUILTIN_COMMANDS;

use super::plugin_entry::PluginEntry;

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
        if let Ok(home) = env::var("HOME") {
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

        self.remove_stale_plugins(&discovered, engine);
        self.load_or_reload_plugins(&discovered, engine, &mut warnings);
        check_conflicts(&self.plugins, &mut warnings);

        warnings
    }

    fn remove_stale_plugins(&mut self, discovered: &HashMap<String, PathBuf>, engine: &mut super::ScriptEngine) {
        let current_names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in &current_names {
            if !discovered.contains_key(name) {
                self.unload_plugin(name, engine);
            }
        }
    }

    fn load_or_reload_plugins(
        &mut self,
        discovered: &HashMap<String, PathBuf>,
        engine: &mut super::ScriptEngine,
        warnings: &mut Vec<String>,
    ) {
        for (name, path) in discovered {
            let mtime = file_mtime(path);
            if let Some(existing) = self.plugins.get(name) {
                if existing.mtime == mtime {
                    continue;
                }
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
                match self.load_plugin_file(name, path, mtime, engine) {
                    Ok(()) => log::info!("plugin loaded: {name}"),
                    Err(e) => {
                        warnings.push(format!("plugin {name}: {e}"));
                        log::warn!("plugin {name} load failed: {e}");
                    }
                }
            }
        }
    }

    /// Discover all plugin init.tcl files across plugin directories.
    fn discover_plugins(&self) -> HashMap<String, PathBuf> {
        let mut found = HashMap::new();
        for dir in &self.plugin_dirs {
            let Ok(entries) = fs::read_dir(dir) else {
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
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

        if let Err(e) = engine.validate(&content) {
            return Err(format!("syntax error: {e}"));
        }

        let before: HashSet<String> = engine.proc_names().into_iter().collect();
        engine.eval(&content).map_err(|e| format!("eval error: {e}"))?;
        let after: HashSet<String> = engine.proc_names().into_iter().collect();
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

fn check_conflicts(plugins: &HashMap<String, PluginEntry>, warnings: &mut Vec<String>) {
    let mut proc_owners: HashMap<&str, &str> = HashMap::new();
    for (name, entry) in plugins {
        for proc_name in &entry.procs {
            if BUILTIN_COMMANDS.contains(&proc_name.as_str()) {
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
}

fn remove_procs(engine: &mut super::ScriptEngine, procs: &[String]) {
    for name in procs {
        engine.remove_proc(name);
    }
}

fn file_mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
