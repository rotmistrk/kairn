//! Application state shared across command handler invocations.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::broker::FileBroker;
use crate::kiro_registry::KiroTabRegistry;
use crate::lsp::registry::LspRegistry;
use crate::message_ring::MessageRing;
use crate::settings::AppSettings;

/// Application state shared across command handler invocations.
pub struct AppState {
    pub broker: FileBroker,
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
    pub theme_state: Option<std::cell::RefCell<crate::app_palette::ThemeState>>,
    pub grep_pending: Option<(String, std::sync::Arc<crate::grep::GrepState>, std::path::PathBuf)>,
    pub build_pending: Option<(
        String,
        std::sync::Arc<crate::task_output::TaskOutput>,
        std::path::PathBuf,
    )>,
    pub pending_tab: Option<crate::eviction::PendingTab>,
    /// Active confirmation context — routes CM_CONFIRM_RESPONSE to the right handler.
    pub confirm_context: Option<crate::commands::ConfirmContext>,
}

impl AppState {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            broker: FileBroker::new(),
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
        }
    }

    pub fn with_settings(root_dir: PathBuf, settings: AppSettings) -> Self {
        Self {
            broker: FileBroker::new(),
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
        }
    }
}
