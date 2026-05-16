//! Application state shared across command handler invocations.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::broker::FileBroker;
use crate::buffer_registry::BufferRegistry;
use crate::kiro_registry::KiroTabRegistry;
use crate::lsp::registry::LspRegistry;
use crate::message_ring::MessageRing;
use crate::scripting::hooks::HookTrigger;
use crate::scripting::ScriptEngine;
use crate::settings::AppSettings;

/// A deferred LSP request waiting for server initialization.
pub struct DeferredLspRequest {
    pub command: txv_core::prelude::CommandId,
    pub data: Box<dyn std::any::Any + Send>,
    pub language: String,
    pub created: std::time::Instant,
}

/// Application state shared across command handler invocations.
pub struct AppState {
    pub broker: FileBroker,
    pub buffers: BufferRegistry,
    pub root_dir: PathBuf,
    pub settings: AppSettings,
    pub lsp: LspRegistry,
    pub(crate) lsp_pending: crate::lsp::handler::PendingRequests,
    pub build_errors: Vec<crate::build::ErrorLocation>,
    pub build_error_idx: usize,
    /// Last known cursor position (0-indexed line, col) from the editor.
    pub cursor_pos: (u32, u32),
    /// Shared message ring buffer.
    pub messages: Arc<Mutex<MessageRing>>,
    /// Registry of active kiro tabs for session persistence.
    pub kiro_registry: KiroTabRegistry,
    /// LSP document version counters (keyed by file path string).
    pub doc_versions: std::collections::HashMap<String, i64>,
    /// MCP snapshot (updated periodically for MCP server reads).
    pub mcp_snapshot: Option<Arc<Mutex<crate::mcp::snapshot::McpSnapshot>>>,
    /// MCP command queue for write operations from MCP tools.
    pub mcp_commands: Option<crate::mcp::commands::McpCommandQueue>,
    pub(crate) mcp_tick: u16,
    pub waker: Option<txv_core::run::Waker>,
    pub theme_state: Option<std::cell::RefCell<crate::theme_state::ThemeState>>,
    pub grep_pending: Option<(String, std::sync::Arc<crate::grep::GrepState>, std::path::PathBuf)>,
    pub build_pending: Option<(
        String,
        std::sync::Arc<crate::task_output::TaskOutput>,
        std::path::PathBuf,
    )>,
    pub pending_tab: Option<crate::eviction::PendingTab>,
    /// Active confirmation context — routes CM_CONFIRM_RESPONSE to the right handler.
    pub confirm_context: Option<crate::commands::ConfirmContext>,
    /// Tcl scripting engine.
    pub script: ScriptEngine,
    /// Pending hook triggers from the editor.
    pub pending_hooks: Vec<HookTrigger>,
    /// Dynamic command list for completions (shared with completer).
    pub command_list: crate::completer::CommandList,
    /// Known LSP language IDs for completions (shared with completer).
    pub lsp_languages: crate::completer::LspLanguageList,
    /// Plugin hot-reload manager.
    pub plugins: crate::scripting::plugins::PluginManager,
    /// Deferred LSP requests waiting for server initialization.
    pub deferred_lsp: Vec<DeferredLspRequest>,
    /// LSP server status tracker (per-language state for status bar).
    pub lsp_status: crate::lsp::progress::LspStatusTracker,
    /// Path of the todo item whose note is currently open in the Notes tab.
    pub todo_note_path: Option<Vec<usize>>,
}

impl AppState {
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
            doc_versions: std::collections::HashMap::new(),
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
            command_list: crate::completer::new_command_list(),
            lsp_languages: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            plugins: crate::scripting::plugins::PluginManager::new(),
            deferred_lsp: Vec::new(),
            lsp_status: crate::lsp::progress::LspStatusTracker::new(),
            todo_note_path: None,
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
            doc_versions: std::collections::HashMap::new(),
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
            command_list: crate::completer::new_command_list(),
            lsp_languages: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            plugins: crate::scripting::plugins::PluginManager::new(),
            deferred_lsp: Vec::new(),
            lsp_status: crate::lsp::progress::LspStatusTracker::new(),
            todo_note_path: None,
        };
        s.lsp_pending.timeout_secs = lsp_timeout;
        s
    }

    /// Returns the syntax theme name appropriate for the current light/dark mode.
    pub fn current_syntax_theme(&self) -> &str {
        let is_light = self
            .theme_state
            .as_ref()
            .map(|ts| ts.borrow().mode == txv_core::palette::ThemeMode::Light)
            .unwrap_or(false);
        self.settings.syntax_theme_for_mode(is_light)
    }
}
