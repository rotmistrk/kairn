//! AppState constructors.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use txv_core::clipboard_ring::new_clipboard;
use txv_core::palette::ThemeMode;
use txv_core::shared_history::SharedHistory;

use crate::completer::new_command_list;
use crate::lsp::pending::PendingRequests;
use crate::lsp::registry::LspRegistry;
use crate::lsp_state::LspState;
use crate::mcp_state::McpState;
use crate::message_ring::MessageRing;
use crate::scripting::plugins::PluginManager;
use crate::scripting::ScriptEngine;
use crate::settings::AppSettings;

use crate::app_state::AppState;
use crate::broker::FileBroker;
use crate::buffer_registry::BufferRegistry;
use crate::handler_context::open_tty_for_title;
use crate::kiro_registry::KiroTabRegistry;
use crate::workspace_roots::WorkspaceRoots;

impl AppState {
    pub fn with_settings(root_dir: PathBuf, settings: AppSettings) -> Self {
        let lsp_pending = PendingRequests::with_timeout(settings.lsp_timeout);
        let clip_max = settings.clipboard_max;
        let mut s = Self {
            roots: WorkspaceRoots::new(root_dir.clone()),
            root_dir,
            settings,
            lsp_pending,
            tab_titles_dirty: true,
            ..Self::empty()
        };
        s.clipboard = new_clipboard(clip_max);
        s.script.set_clipboard(s.clipboard.clone());
        s
    }

    pub(crate) fn empty() -> Self {
        let mut s = Self::core_state();
        s.tty_file = open_tty_for_title();
        s.clipboard = new_clipboard(50);
        s.command_history = SharedHistory::new(100);
        s.search_history = SharedHistory::new(50);
        s.script.set_clipboard(s.clipboard.clone());
        s.messages = Arc::new(Mutex::new(MessageRing::new()));
        s
    }

    fn core_state() -> Self {
        Self {
            broker: FileBroker::new(),
            buffers: BufferRegistry::new(),
            roots: WorkspaceRoots::new(PathBuf::new()),
            root_dir: PathBuf::new(),
            settings: AppSettings::default(),
            lsp: LspRegistry::new(),
            lsp_pending: PendingRequests::with_timeout(5),
            build_errors: Vec::new(),
            build_error_idx: 0,
            cursor_pos: (0, 0),
            messages: Arc::new(Mutex::new(MessageRing::new())),
            kiro_registry: KiroTabRegistry::default(),
            lsp_state: LspState::new(),
            mcp: McpState::default(),
            waker: None,
            theme_state: None,
            grep_pending: None,
            build_pending: None,
            pending_tab: None,
            confirm_context: None,
            script: ScriptEngine::new(None),
            pending_hooks: Vec::new(),
            command_list: new_command_list(),
            lsp_languages: Arc::new(Mutex::new(Vec::new())),
            completer_roots: Arc::new(Mutex::new(Vec::new())),
            plugins: PluginManager::new(),
            todo_note_path: None,
            linked_scroll: false,
            shared_register: Arc::default(),
            clipboard: new_clipboard(1),
            command_history: SharedHistory::new(1),
            search_history: SharedHistory::new(1),
            pty_last_output: HashMap::new(),
            last_window_title: String::new(),
            tty_file: None,
            tab_titles_dirty: false,
            show_messages_on_start: false,
            key_bindings: Vec::new(),
        }
    }

    pub fn current_syntax_theme(&self) -> &str {
        let is_light = self
            .theme_state
            .as_ref()
            .map(|ts| ts.borrow().mode() == ThemeMode::Light)
            .unwrap_or(false);
        self.settings.syntax_theme_for_mode(is_light)
    }
}
