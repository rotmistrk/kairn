//! ScriptCommand enum and StateSnapshot — shared types for the scripting subsystem.

use crate::commands::ViewContext;

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
    Search {
        pattern: Option<String>,
    },
    ClearHighlight,
    SplitVertical {
        file: Option<String>,
    },
    SplitHorizontal {
        file: Option<String>,
    },
    SplitClose,
    SplitFocus,
    SplitOpen {
        path: String,
    },
}
