//! Tcl scripting engine — embeds rusticle interpreter with kairn bridge commands.

mod bridge_build;
mod bridge_editor;
mod bridge_git;
mod bridge_hook;
mod bridge_keymap;
mod bridge_lsp;
mod bridge_system;
mod bridge_todo;
mod bridge_view;
pub mod hooks;
pub mod plugins;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::commands::ViewContext;

use self::hooks::HookRegistry;

/// Read-only snapshot of app state, updated each tick for Tcl queries.
#[derive(Clone, Default)]
pub struct StateSnapshot {
    pub context: ViewContext,
    pub root_dir: String,
    pub selection_text: String,
    pub current_line_text: String,
}

/// Commands produced by Tcl scripts, drained by the handler.
#[derive(Debug)]
pub enum ScriptCommand {
    OpenFile {
        path: String,
        line: Option<u32>,
        col: Option<u32>,
    },
    Save,
    SaveAll,
    Close,
    Goto {
        line: u32,
        col: u32,
    },
    Insert {
        text: String,
    },
    Undo,
    Redo,
    ShowMessage {
        level: String,
        origin: String,
        text: String,
    },
    StatusFlash {
        text: String,
    },
    FocusSlot {
        slot: String,
    },
    RunBuild {
        command: Option<String>,
    },
    RunTest {
        command: Option<String>,
    },
    SetKeyBinding {
        key: String,
        command: String,
    },
    UnbindKey {
        key: String,
    },
    LspHover,
    LspDefinition,
    LspReferences,
    LspRename {
        new_name: String,
    },
    LspFormat,
    GitStage {
        file: String,
    },
    GitUnstage {
        file: String,
    },
    GitCommit {
        message: String,
    },
    GitBlame,
    TodoAdd {
        text: String,
        parent: Option<String>,
    },
    TodoRemove {
        path: String,
    },
    TodoComplete {
        path: String,
    },
    // Editor selection/text operations (Feature 2)
    GetSelection,
    ReplaceSelection {
        text: String,
    },
    GetLine {
        line: Option<u32>,
    },
    DeleteLine {
        line: Option<u32>,
    },
    ReplaceWord {
        text: String,
    },
    // Search highlighting (None pattern = clear)
    Search {
        pattern: Option<String>,
    },
    ClearHighlight,
    DiffRevert,
}

/// The scripting engine: interpreter + command queue + state snapshot.
pub struct ScriptEngine {
    interp: Interpreter,
    commands: Arc<Mutex<Vec<ScriptCommand>>>,
    snapshot: Arc<Mutex<StateSnapshot>>,
    pub hook_registry: Arc<Mutex<HookRegistry>>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let commands: Arc<Mutex<Vec<ScriptCommand>>> = Arc::new(Mutex::new(Vec::new()));
        let snapshot: Arc<Mutex<StateSnapshot>> = Arc::new(Mutex::new(StateSnapshot::default()));
        let hook_registry: Arc<Mutex<HookRegistry>> = Arc::new(Mutex::new(HookRegistry::new()));

        let mut interp = Interpreter::new();
        bridge_editor::register(&mut interp, commands.clone(), snapshot.clone());
        bridge_view::register(&mut interp, commands.clone());
        bridge_system::register(&mut interp, snapshot.clone());
        bridge_build::register(&mut interp, commands.clone());
        bridge_keymap::register(&mut interp, commands.clone());
        bridge_hook::register(&mut interp, hook_registry.clone());
        bridge_lsp::register(&mut interp, commands.clone());
        bridge_git::register(&mut interp, commands.clone());
        bridge_todo::register(&mut interp, commands.clone());

        Self {
            interp,
            commands,
            snapshot,
            hook_registry,
        }
    }

    /// Evaluate a Tcl script. Returns the result or error message.
    pub fn eval(&mut self, script: &str) -> Result<String, String> {
        self.interp
            .eval(script)
            .map(|v| v.as_str().into_owned())
            .map_err(|e| e.message)
    }

    /// Check if a Tcl command is registered.
    pub fn has_command(&self, name: &str) -> bool {
        self.interp.has_command(name)
    }

    /// Get all registered Tcl command names (for completion).
    pub fn command_names(&self) -> Vec<String> {
        self.interp.command_names()
    }

    /// Get all user-defined proc names.
    pub fn proc_names(&self) -> Vec<String> {
        self.interp.proc_names()
    }

    /// Remove a user-defined proc by name.
    pub fn remove_proc(&mut self, name: &str) {
        self.interp.remove_proc(name);
    }

    /// Validate Tcl syntax without executing. Returns Ok(()) or error message.
    pub fn validate(&self, script: &str) -> Result<(), String> {
        let result = self.interp.validate(script);
        if result.is_ok() {
            Ok(())
        } else {
            let msg = result
                .errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            Err(msg)
        }
    }

    /// Load and evaluate a Tcl file.
    pub fn load_file(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        self.eval(&content)?;
        Ok(())
    }

    /// Drain pending commands produced by scripts.
    pub fn drain_commands(&self) -> Vec<ScriptCommand> {
        if let Ok(mut cmds) = self.commands.lock() {
            std::mem::take(&mut *cmds)
        } else {
            Vec::new()
        }
    }

    /// Update the state snapshot (called each tick from handler).
    pub fn update_snapshot(&self, ctx: &ViewContext, root_dir: &str, selection_text: &str, current_line_text: &str) {
        if let Ok(mut snap) = self.snapshot.lock() {
            snap.context = ctx.clone();
            snap.root_dir = root_dir.to_string();
            snap.selection_text = selection_text.to_string();
            snap.current_line_text = current_line_text.to_string();
        }
    }

    /// Get captured output from puts commands.
    pub fn get_output(&self) -> Vec<String> {
        self.interp.get_output().to_vec()
    }

    /// Clear captured output.
    pub fn clear_output(&mut self) {
        self.interp.clear_output();
    }

    /// Load config files in standard order. Errors are logged, never fatal.
    /// Plugins are handled separately by PluginManager.
    pub fn load_config(&mut self, root_dir: &Path) {
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let config = Path::new(&home).join(".kairn/config.tcl");
            if config.exists() {
                if let Err(e) = self.load_file(&config) {
                    log::warn!("config.tcl: {e}");
                }
            }
        }
        let project_init = root_dir.join(".kairn/init.tcl");
        if project_init.exists() {
            if let Err(e) = self.load_file(&project_init) {
                log::warn!("init.tcl: {e}");
            }
        }
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn arg_str(args: &[TclValue], idx: usize) -> Result<String, TclError> {
    args.get(idx)
        .map(|v| v.as_str().into_owned())
        .ok_or_else(|| TclError::new(format!("missing argument {idx}")))
}

pub(crate) fn arg_opt(args: &[TclValue], idx: usize) -> Option<String> {
    args.get(idx).map(|v| v.as_str().into_owned())
}
