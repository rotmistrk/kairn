//! MCP write command queue — allows MCP tools to send mutations to the main thread.

use serde_json::Value;

/// A request from an MCP tool to mutate app state.
pub struct McpRequest {
    pub(crate) action: McpAction,
    pub(crate) reply: std::sync::mpsc::SyncSender<Result<Value, String>>,
}

impl McpRequest {
    pub fn new(action: McpAction, reply: std::sync::mpsc::SyncSender<Result<Value, String>>) -> Self {
        Self { action, reply }
    }
}

/// Actions the MCP server can request.
pub enum McpAction {
    /// Toggle a todo item's completion state. Path is index-based (e.g., [0, 2]).
    TodoToggle {
        path: Vec<usize>,
    },
    /// Add a todo item as sibling after the given path.
    TodoAdd {
        path: Vec<usize>,
        title: String,
    },
    /// Remove a todo item at the given path.
    TodoRemove {
        path: Vec<usize>,
    },
    /// Move a todo item up within its siblings.
    TodoMoveUp {
        path: Vec<usize>,
    },
    /// Move a todo item down within its siblings.
    TodoMoveDown {
        path: Vec<usize>,
    },
    /// Promote a todo item (decrease nesting).
    TodoPromote {
        path: Vec<usize>,
    },
    /// Demote a todo item (increase nesting).
    TodoDemote {
        path: Vec<usize>,
    },
    /// Set the note on a todo item.
    TodoSetNote {
        path: Vec<usize>,
        note: String,
    },
    TodoToggleImportant {
        path: Vec<usize>,
    },
    TodoSetPriority {
        path: Vec<usize>,
        priority: u8,
    },
    TodoSetCompleted {
        path: Vec<usize>,
        state: String,
    },
    TodoSetLoe {
        path: Vec<usize>,
        effort: u8,
    },
    TodoEdit {
        path: Vec<usize>,
        title: String,
    },
    /// Add a subtree of items as children of the item at path.
    TodoAddSubtree {
        path: Vec<usize>,
        items: Vec<serde_json::Value>,
    },
    /// Open an existing file in the editor.
    OpenFile {
        path: String,
    },
    /// Open file and highlight specified line ranges (ephemeral).
    HighlightCode {
        path: String,
        ranges: Vec<(u32, u32)>,
    },
    /// Create a new file on disk and open it.
    CreateFile {
        path: String,
        content: String,
    },
    /// Close an editor tab by path/name.
    CloseTab {
        name: String,
    },
    /// Copy text to clipboard ring.
    ClipboardCopy {
        text: String,
        source: String,
    },
    /// Paste from clipboard ring.
    ClipboardPaste,
    /// List clipboard entries.
    ClipboardList,
    /// Replace lines in an open buffer.
    EditBuffer {
        name: String,
        start_line: usize,
        end_line: usize,
        text: String,
    },
    /// Insert text at a position in an open buffer.
    InsertText {
        name: String,
        line: usize,
        col: usize,
        text: String,
    },
    /// Move cursor to a position in a tab.
    SetCursor {
        name: String,
        line: usize,
        col: usize,
    },
    /// Save the buffer to disk.
    SaveFile {
        name: String,
    },
    /// Get diagnostics for a file.
    GetDiagnostics {
        name: String,
    },
    /// Get last build errors.
    GetBuildErrors,
    /// Search project files (synchronous grep).
    SearchProject {
        pattern: String,
        all_roots: bool,
    },
    /// Trigger a build command.
    RunBuild {
        command: String,
    },
    /// Create a vertical split (optionally with a file in the new pane).
    SplitVertical {
        file: Option<String>,
    },
    /// Create a horizontal split.
    SplitHorizontal {
        file: Option<String>,
    },
    /// Close split, keep focused pane.
    SplitClose,
    /// Switch focus to the other split pane.
    SplitFocus,
    /// Open a file in the other split pane.
    SplitOpen {
        path: String,
    },
    /// Set linked scroll on/off.
    SplitLinked {
        on: bool,
    },
    /// Revert the diff hunk under cursor in the specified tab.
    DiffRevert {
        name: String,
    },
    /// LSP control: start/restart/stop/timeout/args.
    LspControl {
        command: String,
    },
    /// Send input text to a terminal tab.
    SendTerminalInput {
        name: String,
        input: String,
    },
    /// Git stage a file.
    GitStage {
        file: String,
    },
    /// Git unstage a file.
    GitUnstage {
        file: String,
    },
    /// Git commit with message.
    GitCommit {
        message: String,
    },
    /// LSP hover at current cursor.
    LspHover {
        name: String,
    },
    /// LSP go to definition.
    LspDefinition {
        name: String,
    },
    /// LSP find references.
    LspReferences {
        name: String,
    },
    /// LSP rename symbol.
    LspRename {
        name: String,
        new_name: String,
    },
    /// LSP code action.
    LspCodeAction {
        name: String,
    },
    /// LSP format file.
    LspFormat {
        name: String,
    },
    /// Undo in the specified buffer.
    Undo {
        name: String,
    },
    /// Redo in the specified buffer.
    Redo {
        name: String,
    },
    /// Evaluate a Tcl script.
    EvalTcl {
        script: String,
    },
    /// List workspace roots.
    ListRoots,
    /// Add a workspace root.
    AddRoot {
        path: String,
    },
    /// Remove a workspace root.
    RemoveRoot {
        path: String,
    },
    /// Confirm a tool invocation (prompt user, block until response).
    ConfirmTool {
        tool_name: String,
        args_summary: String,
    },
}

pub use super::command_queue::McpCommandQueue;
