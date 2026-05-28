//! ScriptCommand enum and StateSnapshot — shared types for the scripting subsystem.

use crate::commands::ViewContext;

/// Read-only snapshot of app state, updated each tick for Tcl queries.
#[derive(Clone, Default)]
pub struct StateSnapshot {
    pub(crate) context: ViewContext,
    pub(crate) root_dir: String,
    pub(crate) selection_text: String,
    pub(crate) current_line_text: String,
    pub(crate) split_direction: String,
    pub(crate) split_linked: bool,
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
    ViewTheme {
        mode: String,
    },
    ViewZoom,
    ViewToggleTree,
    ViewToggleTools,
    ViewLayout,
    RunBuild {
        command: Option<String>,
    },
    RunTest {
        command: Option<String>,
    },
    TestFile,
    TestAtCursor,
    NextError,
    PrevError,
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
    GitNoBlame,
    GitUntrack {
        file: String,
    },
    GitLog,
    GitDiff,
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
    TodoToggleImportant {
        path: String,
    },
    TodoEdit {
        path: String,
        text: String,
    },
    TodoSwap {
        path: String,
        direction: String,
    },
    TodoPromote {
        path: String,
    },
    TodoDemote {
        path: String,
    },
    TodoList,
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
    SplitLinked {
        on: bool,
    },
    DiffRevert,
    Grep {
        pattern: String,
    },
    LspStart {
        pattern: String,
    },
    LspRestart {
        pattern: String,
    },
    LspStop {
        pattern: String,
    },
    LspTimeout {
        pattern: String,
        secs: Option<u64>,
    },
    LspArgs {
        pattern: String,
        command: String,
    },
    LspEnv {
        pattern: String,
        key: String,
        value: String,
    },
}
