//! Application state shared across command handler invocations.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use txv_core::palette::ThemeMode;
use txv_core::run::Waker;

use crate::broker::FileBroker;
use crate::buffer_registry::BufferRegistry;
use crate::build::ErrorLocation;
use crate::commands::ConfirmContext;
use crate::completer::{new_command_list, CommandList, LspLanguageList};
use crate::deferred_lsp_request::DeferredLspRequest;
use crate::desktop::SlotId;
use crate::eviction::PendingTab;
use crate::grep::GrepState;
use crate::kiro_registry::KiroTabRegistry;
use crate::lsp::progress::LspStatusTracker;
use crate::lsp::registry::LspRegistry;
use crate::mcp::commands::McpCommandQueue;
use crate::mcp::snapshot::McpSnapshot;
use crate::message_ring::MessageRing;
use crate::scripting::hooks::HookTrigger;
use crate::scripting::plugins::PluginManager;
use crate::scripting::ScriptEngine;
use crate::settings::AppSettings;
use crate::task_output::TaskOutput;
use crate::theme_state::ThemeState;

/// Application state shared across command handler invocations.
pub struct AppState {
    pub(crate) broker: FileBroker,
    pub(crate) buffers: BufferRegistry,
    pub(crate) root_dir: PathBuf,
    pub(crate) settings: AppSettings,
    pub(crate) lsp: LspRegistry,
    pub(crate) lsp_pending: crate::lsp::handler::PendingRequests,
    pub(crate) build_errors: Vec<ErrorLocation>,
    pub(crate) build_error_idx: usize,
    /// Last known cursor position (0-indexed line, col) from the editor.
    pub(crate) cursor_pos: (u32, u32),
    /// Shared message ring buffer.
    pub(crate) messages: Arc<Mutex<MessageRing>>,
    /// Registry of active kiro tabs for session persistence.
    pub(crate) kiro_registry: KiroTabRegistry,
    /// LSP document version counters (keyed by file path string).
    pub(crate) doc_versions: HashMap<String, i64>,
    pub(crate) lsp_opened_files: HashSet<String>,
    /// MCP snapshot (updated periodically for MCP server reads).
    pub(crate) mcp_snapshot: Option<Arc<Mutex<McpSnapshot>>>,
    /// MCP command queue for write operations from MCP tools.
    pub(crate) mcp_commands: Option<McpCommandQueue>,
    pub(crate) mcp_tick: u16,
    pub(crate) waker: Option<Waker>,
    pub(crate) theme_state: Option<RefCell<ThemeState>>,
    pub(crate) grep_pending: Option<(String, Arc<GrepState>, PathBuf)>,
    pub(crate) build_pending: Option<(String, Arc<TaskOutput>, PathBuf)>,
    pub(crate) pending_tab: Option<PendingTab>,
    /// Active confirmation context — routes CM_CONFIRM_RESPONSE to the right handler.
    pub(crate) confirm_context: Option<ConfirmContext>,
    /// Tcl scripting engine.
    pub(crate) script: ScriptEngine,
    /// Pending hook triggers from the editor.
    pub(crate) pending_hooks: Vec<HookTrigger>,
    /// Dynamic command list for completions (shared with completer).
    pub(crate) command_list: CommandList,
    /// Known LSP language IDs for completions (shared with completer).
    pub(crate) lsp_languages: LspLanguageList,
    /// Plugin hot-reload manager.
    pub(crate) plugins: PluginManager,
    /// Deferred LSP requests waiting for server initialization.
    pub(crate) deferred_lsp: Vec<DeferredLspRequest>,
    /// LSP server status tracker (per-language state for status bar).
    pub(crate) lsp_status: LspStatusTracker,
    /// Path of the todo item whose note is currently open in the Notes tab.
    pub(crate) todo_note_path: Option<Vec<usize>>,
    /// Whether the center panel's split has linked scrolling enabled.
    pub(crate) linked_scroll: bool,
    /// Last output timestamp per terminal tab index (for activity badges).
    pub(crate) pty_last_output: HashMap<usize, Instant>,
}

impl AppState {
    pub fn broker_open(&mut self, path: &str, slot: SlotId, idx: usize) -> crate::broker::OpenResult {
        self.broker.open(path, slot, idx)
    }
    pub fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }
    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }
    pub fn lsp_set_waker(&mut self, waker: Waker) {
        self.lsp.set_waker(waker);
    }
    pub fn lsp_shutdown_all(&mut self) {
        self.lsp.shutdown_all();
    }
    pub fn set_mcp_snapshot(&mut self, snap: Arc<Mutex<McpSnapshot>>) {
        self.mcp_snapshot = Some(snap);
    }
    pub fn mcp_snapshot(&self) -> &Option<Arc<Mutex<McpSnapshot>>> {
        &self.mcp_snapshot
    }
    pub fn mcp_commands(&self) -> &Option<McpCommandQueue> {
        &self.mcp_commands
    }
    pub fn set_mcp_commands(&mut self, q: McpCommandQueue) {
        self.mcp_commands = Some(q);
    }
    pub fn messages(&self) -> &Arc<Mutex<MessageRing>> {
        &self.messages
    }
    pub fn set_waker(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }
    pub fn set_theme_state(&mut self, ts: ThemeState) {
        self.theme_state = Some(RefCell::new(ts));
    }
    pub fn script(&self) -> &ScriptEngine {
        &self.script
    }
    pub fn script_mut(&mut self) -> &mut ScriptEngine {
        &mut self.script
    }
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        self.plugins.add_plugin_dir(dir);
    }
    pub fn command_list(&self) -> &CommandList {
        &self.command_list
    }
    pub fn lsp_languages(&self) -> &LspLanguageList {
        &self.lsp_languages
    }
    pub fn kiro_registry(&self) -> &KiroTabRegistry {
        &self.kiro_registry
    }
    pub fn kiro_registry_mut(&mut self) -> &mut KiroTabRegistry {
        &mut self.kiro_registry
    }
    pub fn pending_tab(&self) -> &Option<PendingTab> {
        &self.pending_tab
    }
    pub fn set_pending_tab(&mut self, tab: Option<PendingTab>) {
        self.pending_tab = tab;
    }
    pub fn todo_note_path(&self) -> &Option<Vec<usize>> {
        &self.todo_note_path
    }
    pub fn set_todo_note_path(&mut self, path: Option<Vec<usize>>) {
        self.todo_note_path = path;
    }
    pub fn record_pty_output(&mut self, index: usize, when: Instant) {
        self.pty_last_output.insert(index, when);
    }

    pub fn refresh_plugins(&mut self) -> Vec<String> {
        self.plugins.refresh(&mut self.script)
    }

    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            broker: FileBroker::new(),
            buffers: BufferRegistry::new(),
            root_dir,
            settings: AppSettings::default(),
            lsp: LspRegistry::new(),
            lsp_pending: Default::default(),
            build_errors: Vec::new(),
            build_error_idx: 0,
            cursor_pos: (0, 0),
            messages: Arc::new(Mutex::new(MessageRing::new())),
            kiro_registry: KiroTabRegistry::default(),
            doc_versions: HashMap::new(),
            lsp_opened_files: HashSet::new(),
            mcp_snapshot: None,
            mcp_commands: None,
            mcp_tick: 0,
            waker: None,
            theme_state: None,
            grep_pending: None,
            build_pending: None,
            pending_tab: None,
            confirm_context: None,
            script: ScriptEngine::new(),
            pending_hooks: Vec::new(),
            command_list: new_command_list(),
            lsp_languages: Arc::new(Mutex::new(Vec::new())),
            plugins: PluginManager::new(),
            deferred_lsp: Vec::new(),
            lsp_status: LspStatusTracker::new(),
            todo_note_path: None,
            linked_scroll: false,
            pty_last_output: HashMap::new(),
        }
    }

    pub fn with_settings(root_dir: PathBuf, settings: AppSettings) -> Self {
        let lsp_timeout = settings.lsp_timeout;
        let mut s = Self {
            broker: FileBroker::new(),
            buffers: BufferRegistry::new(),
            root_dir,
            settings,
            lsp: LspRegistry::new(),
            lsp_pending: Default::default(),
            build_errors: Vec::new(),
            build_error_idx: 0,
            cursor_pos: (0, 0),
            messages: Arc::new(Mutex::new(MessageRing::new())),
            kiro_registry: KiroTabRegistry::default(),
            doc_versions: HashMap::new(),
            lsp_opened_files: HashSet::new(),
            mcp_snapshot: None,
            mcp_commands: None,
            mcp_tick: 0,
            waker: None,
            theme_state: None,
            grep_pending: None,
            build_pending: None,
            pending_tab: None,
            confirm_context: None,
            script: ScriptEngine::new(),
            pending_hooks: Vec::new(),
            command_list: new_command_list(),
            lsp_languages: Arc::new(Mutex::new(Vec::new())),
            plugins: PluginManager::new(),
            deferred_lsp: Vec::new(),
            lsp_status: LspStatusTracker::new(),
            todo_note_path: None,
            linked_scroll: false,
            pty_last_output: HashMap::new(),
        };
        s.lsp_pending.timeout_secs = lsp_timeout;
        s
    }

    /// Returns the syntax theme name appropriate for the current light/dark mode.
    pub fn current_syntax_theme(&self) -> &str {
        let is_light = self
            .theme_state
            .as_ref()
            .map(|ts| ts.borrow().mode == ThemeMode::Light)
            .unwrap_or(false);
        self.settings.syntax_theme_for_mode(is_light)
    }
}
