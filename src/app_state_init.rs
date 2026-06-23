//! AppState constructors.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use txv_core::clipboard_ring::new_clipboard;
use txv_core::palette::ThemeMode;
use txv_core::shared_history::SharedHistory;

use crate::completer::new_command_list;
use crate::editor_shared::EditorShared;
use crate::lsp::pending::PendingRequests;
use crate::lsp_subsystem::LspSubsystem;
use crate::mcp_state::McpState;
use crate::message_ring::MessageRing;
use crate::pending_ops::PendingOps;
use crate::script_state::ScriptState;
use crate::scripting::ScriptEngine;
use crate::settings::AppSettings;

use crate::app_state::AppState;
use crate::build_state::BuildState;
use crate::handler_context::open_tty_for_title;
use crate::kiro_registry::KiroTabRegistry;
use crate::ui_chrome::UiChrome;
use crate::workspace_state::WorkspaceState;

impl AppState {
    pub fn with_settings(root_dir: PathBuf, settings: AppSettings) -> Self {
        let lsp_timeout = settings.lsp_timeout;
        let clip_max = settings.clipboard_max;
        let langs = Arc::new(Mutex::new(Vec::new()));
        let mut s = Self {
            workspace: WorkspaceState::new(root_dir, settings),
            lsp: LspSubsystem::new(PendingRequests::with_timeout(lsp_timeout), langs),
            ..Self::empty()
        };
        s.ui.set_tab_titles_dirty(true);
        let clip = new_clipboard(clip_max);
        s.editor = EditorShared::new(clip.clone(), SharedHistory::new(100), SharedHistory::new(50));
        s.scripting.script_mut().set_clipboard(clip);
        s
    }

    pub(crate) fn empty() -> Self {
        let mut s = Self::core_state();
        s.ui.set_tty_file(open_tty_for_title());
        let clip = new_clipboard(50);
        s.editor = EditorShared::new(clip.clone(), SharedHistory::new(100), SharedHistory::new(50));
        s.scripting.script_mut().set_clipboard(clip);
        s.messages = Arc::new(Mutex::new(MessageRing::new()));
        s
    }

    fn core_state() -> Self {
        let langs = Arc::new(Mutex::new(Vec::new()));
        Self {
            workspace: WorkspaceState::new(PathBuf::new(), AppSettings::default()),
            lsp: LspSubsystem::new(PendingRequests::with_timeout(5), langs),
            build: BuildState::new(),
            cursor_pos: (0, 0),
            messages: Arc::new(Mutex::new(MessageRing::new())),
            kiro_registry: KiroTabRegistry::default(),
            mcp: McpState::default(),
            pending: PendingOps::new(),
            scripting: ScriptState::new(
                ScriptEngine::new(None),
                new_command_list(),
                Arc::new(Mutex::new(Vec::new())),
            ),
            editor: EditorShared::new(new_clipboard(1), SharedHistory::new(1), SharedHistory::new(1)),
            ui: UiChrome::new(),
            diff_base: HashMap::new(),
        }
    }

    pub fn current_syntax_theme(&self) -> &str {
        let is_light = self
            .ui
            .theme_state()
            .as_ref()
            .map(|ts| ts.borrow().mode() == ThemeMode::Light)
            .unwrap_or(false);
        self.workspace.settings().syntax_theme_for_mode(is_light)
    }
}
