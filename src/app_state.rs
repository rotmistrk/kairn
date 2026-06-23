//! Application state shared across command handler invocations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::build_state::BuildState;
use crate::desktop::SlotId;
use crate::editor_shared::EditorShared;
use crate::eviction::PendingTab;
use crate::kiro_registry::KiroTabRegistry;
use crate::lsp_subsystem::LspSubsystem;
use crate::mcp::commands::McpCommandQueue;
use crate::mcp::snapshot::McpSnapshot;
use crate::mcp_state::McpState;
use crate::message_ring::MessageRing;
use crate::pending_ops::PendingOps;
use crate::script_state::ScriptState;
use crate::theme_state::ThemeState;
use crate::ui_chrome::UiChrome;

use crate::workspace_state::WorkspaceState;

/// Application state shared across command handler invocations.
pub struct AppState {
    pub(crate) workspace: WorkspaceState,
    pub(crate) lsp: LspSubsystem,
    pub(crate) build: BuildState,
    /// Last known cursor position (0-indexed line, col) from the editor.
    pub(crate) cursor_pos: (u32, u32),
    /// Shared message ring buffer.
    pub(crate) messages: Arc<Mutex<MessageRing>>,
    /// Registry of active kiro tabs for session persistence.
    pub(crate) kiro_registry: KiroTabRegistry,
    /// MCP state (snapshot, command queue, tick counter).
    pub(crate) mcp: McpState,
    /// Pending async operations.
    pub(crate) pending: PendingOps,
    /// Scripting subsystem (engine, hooks, plugins, completers).
    pub(crate) scripting: ScriptState,
    /// Shared editor state (register, clipboard, histories, linked scroll).
    pub(crate) editor: EditorShared,
    /// UI chrome state (waker, tty, titles, theme, keys).
    pub(crate) ui: UiChrome,
    /// Git diff base commits per root (short hash). When set, git pane shows diff vs this commit.
    pub(crate) diff_base: HashMap<PathBuf, String>,
}

impl AppState {
    pub(crate) fn workspace(&self) -> &WorkspaceState {
        &self.workspace
    }

    pub(crate) fn workspace_mut(&mut self) -> &mut WorkspaceState {
        &mut self.workspace
    }

    pub(crate) fn build(&self) -> &BuildState {
        &self.build
    }

    pub(crate) fn build_mut(&mut self) -> &mut BuildState {
        &mut self.build
    }

    pub fn broker_open(&mut self, path: &str, slot: SlotId, idx: usize) -> crate::broker::OpenResult {
        self.workspace.broker_mut().open(path, slot, idx)
    }
    pub fn broker_is_open(&self, path: &str) -> bool {
        self.workspace.broker().is_open(path)
    }
    pub fn broker_open_count(&self) -> usize {
        self.workspace.broker().open_paths().len()
    }
    pub fn root_dir(&self) -> &PathBuf {
        self.workspace.root_dir()
    }

    /// Access workspace roots.
    pub fn roots(&self) -> &crate::workspace_roots::WorkspaceRoots {
        self.workspace.roots()
    }

    /// Mutable access to workspace roots.
    pub fn roots_mut(&mut self) -> &mut crate::workspace_roots::WorkspaceRoots {
        self.workspace.roots_mut()
    }
    pub fn clipboard_ref(&self) -> &txv_core::clipboard_ring::ClipboardHandle {
        self.editor.clipboard()
    }

    pub(crate) fn editor(&self) -> &EditorShared {
        &self.editor
    }

    pub(crate) fn editor_mut(&mut self) -> &mut EditorShared {
        &mut self.editor
    }
    pub fn settings(&self) -> &crate::settings::AppSettings {
        self.workspace.settings()
    }
    pub fn settings_mut(&mut self) -> &mut crate::settings::AppSettings {
        self.workspace.settings_mut()
    }
    pub(crate) fn lsp_sub(&self) -> &LspSubsystem {
        &self.lsp
    }

    pub(crate) fn lsp_sub_mut(&mut self) -> &mut LspSubsystem {
        &mut self.lsp
    }

    pub fn lsp_set_waker(&mut self, waker: txv_core::run::Waker) {
        self.lsp.registry_mut().set_waker(waker);
    }
    pub fn lsp_shutdown_all(&mut self) {
        self.lsp.registry_mut().shutdown_all();
    }
    pub fn set_mcp_snapshot(&mut self, snap: Arc<Mutex<McpSnapshot>>) {
        self.mcp.set_snapshot(snap);
    }
    pub fn mcp_snapshot(&self) -> &Option<Arc<Mutex<McpSnapshot>>> {
        self.mcp.snapshot()
    }
    pub fn mcp_commands(&self) -> &Option<McpCommandQueue> {
        self.mcp.commands()
    }
    pub fn set_mcp_commands(&mut self, q: McpCommandQueue) {
        self.mcp.set_commands(q);
    }

    pub(crate) fn mcp(&self) -> &crate::mcp_state::McpState {
        &self.mcp
    }

    pub(crate) fn mcp_mut(&mut self) -> &mut crate::mcp_state::McpState {
        &mut self.mcp
    }
    pub fn messages(&self) -> &Arc<Mutex<MessageRing>> {
        &self.messages
    }
    pub fn set_waker(&mut self, waker: txv_core::run::Waker) {
        self.ui.set_waker(waker);
    }
    pub fn set_theme_state(&mut self, ts: ThemeState) {
        self.ui.set_theme_state(ts);
    }

    pub(crate) fn ui(&self) -> &UiChrome {
        &self.ui
    }

    pub(crate) fn ui_mut(&mut self) -> &mut UiChrome {
        &mut self.ui
    }
    pub(crate) fn scripting(&self) -> &ScriptState {
        &self.scripting
    }
    pub(crate) fn scripting_mut(&mut self) -> &mut ScriptState {
        &mut self.scripting
    }
    pub fn script(&self) -> &crate::scripting::ScriptEngine {
        self.scripting.script()
    }
    pub fn script_mut(&mut self) -> &mut crate::scripting::ScriptEngine {
        self.scripting.script_mut()
    }
    pub fn set_key_bindings(&mut self, b: Vec<txv_core::key_help::KeyHelpEntry>) {
        self.ui.set_key_bindings(b);
    }
    pub fn key_bindings(&self) -> &[txv_core::key_help::KeyHelpEntry] {
        self.ui.key_bindings()
    }
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        self.scripting.plugins_mut().add_plugin_dir(dir);
    }
    pub fn command_list(&self) -> &crate::completer::CommandList {
        self.scripting.command_list()
    }
    pub fn lsp_languages(&self) -> &crate::completer::LspLanguageList {
        self.lsp.languages()
    }
    pub fn completer_roots(&self) -> &crate::completer::RootsList {
        self.scripting.completer_roots()
    }
    pub fn kiro_registry(&self) -> &KiroTabRegistry {
        &self.kiro_registry
    }
    pub fn kiro_registry_mut(&mut self) -> &mut KiroTabRegistry {
        &mut self.kiro_registry
    }
    pub fn pending_tab(&self) -> &Option<PendingTab> {
        self.pending.pending_tab()
    }
    pub fn set_pending_tab(&mut self, tab: Option<PendingTab>) {
        self.pending.set_pending_tab(tab);
    }
    pub fn todo_note_path(&self) -> &Option<Vec<usize>> {
        self.pending.todo_note_path()
    }
    pub fn set_todo_note_path(&mut self, path: Option<Vec<usize>>) {
        self.pending.set_todo_note_path(path);
    }

    pub(crate) fn pending(&self) -> &PendingOps {
        &self.pending
    }

    pub(crate) fn pending_mut(&mut self) -> &mut PendingOps {
        &mut self.pending
    }
    pub fn record_pty_output(&mut self, index: usize, when: std::time::Instant) {
        self.ui.record_pty_output(index, when);
    }

    pub fn last_window_title(&self) -> &str {
        self.ui.last_window_title()
    }

    pub fn refresh_plugins(&mut self) -> Vec<String> {
        self.scripting.refresh_plugins()
    }
}
