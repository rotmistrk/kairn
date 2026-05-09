/// Every operation the editor can perform.
///
/// Keyboard layouts translate input events into these commands.
/// The [`super::Editor`] struct executes them against the buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // ── Cursor movement ──
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,
    GotoLine(usize),
    BracketMatch,

    // ── Editing ──
    InsertChar(char),
    InsertNewline,
    DeleteCharForward,
    DeleteCharBackward,
    DeleteWord,
    DeleteWordBackward,
    DeleteLine,
    DeleteToLineEnd,
    DeleteToLineStart,
    NewlineBelow,
    NewlineAbove,
    Indent,
    Dedent,
    JoinLines,

    // ── Undo/redo ──
    Undo,
    Redo,

    // ── Selection ──
    SelectionStart,
    SelectionLineStart,
    SelectionBlockStart,
    SelectionCancel,
    SelectAll,

    // ── Clipboard ──
    Yank,
    YankLine,
    Paste,
    PasteBefore,

    // ── Search ──
    SearchForward(String),
    SearchBackward(String),
    SearchNext,
    SearchPrev,
    SearchWordUnderCursor,
    ClearSearchHighlight,

    // ── File ──
    Save,
    SaveAs(String),
    SaveAll,
    OpenFile(String),
    CloseBuffer,
    ForceCloseBuffer,

    // ── Mode (vim-specific, other layouts can ignore) ──
    EnterInsertMode,
    EnterInsertAfter,
    EnterInsertLineStart,
    EnterInsertLineEnd,
    EnterInsertBelow,
    EnterInsertAbove,
    ExitInsertMode,
    EnterCommandMode,

    // ── Ex commands ──
    #[allow(clippy::enum_variant_names)]
    ExCommand(String),

    // ── Navigation ──
    GotoDefinition,
    GotoReferences,

    // ── Panel ──
    FocusTree,
    FocusEditor,
    FocusControl,
    FocusBottom,
    FocusNext,
    FocusPrev,
    ToggleTree,
    ToggleControl,
    ToggleBottom,
    CycleLayout,
    CycleTreeMode,
    CycleBottomTab,
    NextTab,
    PrevTab,

    // ── Kiro ──
    SendToKiro,
    SendToKiroWithPrompt(String),

    // ── Application ──
    #[allow(clippy::enum_variant_names)]
    CommandPalette,
    FuzzyFileSearch,
    ContentSearch,
    ToggleBorderMode,
    Quit,
    ForceQuit,

    // ── No-op ──
    Noop,
}

/// Editor mode — determines how keys are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Vim normal / emacs default / classic default.
    Normal,
    /// Vim insert mode.
    Insert,
    /// Vim visual mode (stream, line, or block).
    Visual(VisualKind),
    /// Vim command-line mode (`:` prompt).
    CommandLine,
}

/// Visual selection variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualKind {
    Stream,
    Line,
    Block,
}

/// Result of executing a command — tells the panel what happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    /// Nothing to report.
    None,
    /// Cursor moved — panel should ensure it's visible.
    CursorMoved,
    /// Content changed — panel should re-render.
    ContentChanged,
    /// File should be saved.
    SaveRequested,
    /// Buffer should be closed.
    CloseRequested,
    /// Close blocked — buffer is modified.
    CloseBlocked,
    /// Force close.
    ForceCloseRequested,
    /// Open a file.
    OpenFile(String),
    /// Go to definition of word under cursor.
    GotoDefinition(String),
    /// Find references to word under cursor.
    GotoReferences(String),
    /// Show ex-command output.
    ExOutput(String),
    /// Send text to Kiro.
    SendToKiro(String),
    /// Panel focus change.
    FocusChange(FocusTarget),
}

/// Target for panel focus changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Tree,
    Editor,
    Control,
    Bottom,
    Next,
    Prev,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_variants_constructible() {
        let cmds: Vec<Command> = vec![
            Command::MoveLeft,
            Command::InsertChar('a'),
            Command::GotoLine(42),
            Command::SearchForward("test".into()),
            Command::ExCommand(":w".into()),
            Command::Noop,
        ];
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn editor_mode_equality() {
        assert_eq!(EditorMode::Normal, EditorMode::Normal);
        assert_ne!(EditorMode::Normal, EditorMode::Insert);
        assert_eq!(
            EditorMode::Visual(VisualKind::Line),
            EditorMode::Visual(VisualKind::Line)
        );
    }

    #[test]
    fn editor_action_variants() {
        let a = EditorAction::FocusChange(FocusTarget::Tree);
        assert_eq!(a, EditorAction::FocusChange(FocusTarget::Tree));
    }
}
