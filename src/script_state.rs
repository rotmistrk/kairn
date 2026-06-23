//! Scripting subsystem state: engine, hooks, plugins, completions.

use crate::completer::{CommandList, RootsList};
use crate::scripting::hooks::HookTrigger;
use crate::scripting::plugins::PluginManager;
use crate::scripting::ScriptEngine;

/// Scripting subsystem state.
pub(crate) struct ScriptState {
    script: ScriptEngine,
    pending_hooks: Vec<HookTrigger>,
    plugins: PluginManager,
    command_list: CommandList,
    completer_roots: RootsList,
}

impl ScriptState {
    pub(crate) fn new(script: ScriptEngine, command_list: CommandList, completer_roots: RootsList) -> Self {
        Self {
            script,
            pending_hooks: Vec::new(),
            plugins: PluginManager::new(),
            command_list,
            completer_roots,
        }
    }

    pub(crate) fn script(&self) -> &ScriptEngine {
        &self.script
    }

    pub(crate) fn script_mut(&mut self) -> &mut ScriptEngine {
        &mut self.script
    }

    pub(crate) fn pending_hooks(&self) -> &[HookTrigger] {
        &self.pending_hooks
    }

    pub(crate) fn pending_hooks_mut(&mut self) -> &mut Vec<HookTrigger> {
        &mut self.pending_hooks
    }

    pub(crate) fn plugins_mut(&mut self) -> &mut PluginManager {
        &mut self.plugins
    }

    pub(crate) fn command_list(&self) -> &CommandList {
        &self.command_list
    }

    pub(crate) fn completer_roots(&self) -> &RootsList {
        &self.completer_roots
    }

    /// Refresh plugins and return warnings.
    pub(crate) fn refresh_plugins(&mut self) -> Vec<String> {
        self.plugins.refresh(&mut self.script)
    }
}
