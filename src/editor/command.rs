//! Command — every operation the editor can perform.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Cursor movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    MoveLineStart,
    MoveLineEnd,
    MoveFirstNonBlank,
    MoveFileStart,
    MoveFileEnd,
    GotoLine(usize),
    HalfPageUp,
    HalfPageDown,
    PageUp,
    PageDown,
    MatchBracket,

    // Find char on line
    FindChar(char),
    FindCharBack(char),
    TillChar(char),
    TillCharBack(char),
    RepeatFind,
    RepeatFindReverse,

    // Editing
    InsertChar(char),
    InsertNewline,
    DeleteCharForward,
    DeleteCharBackward,
    DeleteLine,
    DeleteWord,
    DeleteToEnd,
    ChangeWord,
    ChangeLine,
    ChangeToEnd,
    Substitute,
    SubstituteLine,
    NewlineBelow,
    NewlineAbove,
    JoinLines,
    ToggleCase,
    ReplaceChar(char),
    Indent,
    Unindent,

    // Operators (pending motion)
    OperatorDelete,
    OperatorChange,
    OperatorYank,

    // Undo/redo
    Undo,
    Redo,

    // Clipboard
    YankLine,
    Paste,
    PasteBefore,

    // Mode
    EnterInsertMode,
    EnterInsertAfter,
    EnterInsertLineEnd,
    EnterInsertLineStart,
    EnterInsertBelow,
    EnterInsertAbove,
    ExitInsertMode,
    EnterVisual,
    EnterVisualLine,
    ExitVisual,

    // Visual mode operations
    VisualDelete,
    VisualYank,
    VisualIndent,
    VisualUnindent,

    // Search
    SearchForward(String),
    SearchBackward(String),
    SearchNext,
    SearchPrev,
    SearchWordForward,
    SearchWordBackward,
    EnterSearchMode,

    // Ex / command mode
    EnterCommandMode,
    ExCommand(String),

    // File
    Save,
    CloseBuffer,

    // Repeat
    DotRepeat,

    // No-op
    Noop,
}
