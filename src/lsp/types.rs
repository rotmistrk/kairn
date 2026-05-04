//! LSP domain types: semantic wrappers, server config, events, and data types.

use std::collections::{HashMap, HashSet};

use crate::lsp::protocol::{LspError, RequestId};

// ── Semantic wrapper types ──────────────────────────────────

/// Language identifier (matches LSP `languageId`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageId(String);

impl LanguageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Document URI (file:// scheme).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentUri(String);

impl DocumentUri {
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Create from a filesystem path.
    pub fn from_path(path: &str) -> Self {
        Self(format!("file://{path}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract the filesystem path (strips `file://` prefix).
    pub fn to_path(&self) -> Option<&str> {
        self.0.strip_prefix("file://")
    }
}

impl std::fmt::Display for DocumentUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Document version — monotonically increasing per document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentVersion(i32);

impl DocumentVersion {
    pub fn new(v: i32) -> Self {
        Self(v)
    }

    pub fn value(self) -> i32 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

// ── Server configuration ────────────────────────────────────

/// Configuration for a single language server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub language_id: LanguageId,
    pub command: String,
    pub args: Vec<String>,
    pub extensions: Vec<String>,
    pub init_options: Option<serde_json::Value>,
    pub root_markers: Vec<String>,
}

/// Server lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Not started yet.
    Idle,
    /// Initialize request sent, waiting for response.
    Starting,
    /// Running and ready for requests.
    Ready,
    /// Shutdown request sent, waiting for response.
    ShuttingDown,
    /// Crashed or exited unexpectedly.
    Crashed,
    /// Permanently disabled after repeated crashes.
    Disabled,
}

/// Tracks a pending request awaiting a response.
#[derive(Debug)]
pub struct PendingRequest {
    pub method: String,
    pub sent_at: std::time::Instant,
}

// ── Events from server tasks to main thread ─────────────────

/// Events delivered to the main thread from LSP server tasks.
#[derive(Debug)]
pub enum LspEvent {
    /// Server finished initialization and is ready.
    ServerReady(LanguageId),
    /// Server crashed or exited unexpectedly.
    ServerCrashed(LanguageId, String),
    /// Response to a client request.
    Response {
        language_id: LanguageId,
        id: RequestId,
        result: Result<serde_json::Value, LspError>,
    },
    /// Completion results.
    Completions {
        uri: DocumentUri,
        items: Vec<CompletionItem>,
    },
    /// Hover content.
    Hover { uri: DocumentUri, contents: String },
    /// Definition location(s).
    Definition {
        uri: DocumentUri,
        locations: Vec<Location>,
    },
    /// Reference locations.
    References {
        uri: DocumentUri,
        locations: Vec<Location>,
    },
    /// Document symbols.
    Symbols {
        uri: DocumentUri,
        symbols: Vec<DocumentSymbol>,
    },
    /// Formatting edits.
    FormattingEdits {
        uri: DocumentUri,
        edits: Vec<TextEditRange>,
    },
    /// Workspace edits (from rename, code actions).
    WorkspaceEdit {
        edits: HashMap<DocumentUri, Vec<TextEditRange>>,
    },
    /// Diagnostics published by the server.
    Diagnostics {
        uri: DocumentUri,
        diagnostics: Vec<Diagnostic>,
    },
    /// Server message (info, warning, error).
    ShowMessage {
        level: MessageLevel,
        message: String,
    },
}

// ── LSP data types ──────────────────────────────────────────

/// A completion item returned by the server.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub insert_text: String,
}

/// Completion item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Function,
    Method,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Keyword,
    Snippet,
    Other,
}

/// A source location (file + position).
#[derive(Debug, Clone)]
pub struct Location {
    pub uri: DocumentUri,
    pub line: u32,
    pub character: u32,
}

/// A document symbol with optional children.
#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range_start_line: u32,
    pub range_end_line: u32,
    pub children: Vec<DocumentSymbol>,
}

/// Symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Struct,
    Enum,
    Field,
    Constant,
    Variable,
    Module,
    Other,
}

/// A text edit with range.
#[derive(Debug, Clone)]
pub struct TextEditRange {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub new_text: String,
}

/// A diagnostic (error, warning, etc.) from the server.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Server message level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Error,
    Warning,
    Info,
    Log,
}

/// Per-document debounce state for batching changes.
pub struct DebounceState {
    pub pending_changes: Vec<crate::buffer::TextChange>,
    pub last_edit: std::time::Instant,
    pub version: DocumentVersion,
}

// ── Open document tracking ──────────────────────────────────

/// Tracks which documents are open on a given server.
#[derive(Debug, Default)]
pub struct OpenDocSet {
    docs: HashSet<DocumentUri>,
}

impl OpenDocSet {
    pub fn insert(&mut self, uri: DocumentUri) -> bool {
        self.docs.insert(uri)
    }

    pub fn remove(&mut self, uri: &DocumentUri) -> bool {
        self.docs.remove(uri)
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    pub fn contains(&self, uri: &DocumentUri) -> bool {
        self.docs.contains(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_id_display() {
        let id = LanguageId::new("rust");
        assert_eq!(id.as_str(), "rust");
        assert_eq!(format!("{id}"), "rust");
    }

    #[test]
    fn document_uri_from_path() {
        let uri = DocumentUri::from_path("/home/user/main.rs");
        assert_eq!(uri.as_str(), "file:///home/user/main.rs");
        assert_eq!(uri.to_path(), Some("/home/user/main.rs"));
    }

    #[test]
    fn document_version_increments() {
        let v0 = DocumentVersion::new(0);
        let v1 = v0.next();
        assert_eq!(v1.value(), 1);
        assert!(v1 > v0);
    }

    #[test]
    fn open_doc_set_operations() {
        let mut docs = OpenDocSet::default();
        let uri = DocumentUri::from_path("/tmp/test.rs");
        assert!(docs.is_empty());
        assert!(docs.insert(uri.clone()));
        assert!(docs.contains(&uri));
        assert!(!docs.is_empty());
        assert!(docs.remove(&uri));
        assert!(docs.is_empty());
    }

    #[test]
    fn server_state_equality() {
        assert_eq!(ServerState::Idle, ServerState::Idle);
        assert_ne!(ServerState::Idle, ServerState::Ready);
    }

    #[test]
    fn completion_kind_copy() {
        let k = CompletionKind::Function;
        let k2 = k;
        assert_eq!(k, k2);
    }

    #[test]
    fn diagnostic_severity_equality() {
        assert_eq!(DiagnosticSeverity::Error, DiagnosticSeverity::Error);
        assert_ne!(DiagnosticSeverity::Error, DiagnosticSeverity::Warning);
    }
}
