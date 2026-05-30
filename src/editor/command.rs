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
    DeleteWordBackward,
    DeleteToEnd,
    DeleteToStart,
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
    YankWord,
    YankToEnd,
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
    EnterVisualBlock,
    ExitVisual,

    // Visual mode operations
    VisualDelete,
    VisualYank,
    VisualChange,
    VisualIndent,
    VisualUnindent,
    VisualExCommand,
    BlockInsert,
    BlockAppend,
    BlockReplace(char),

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

    // LSP
    GotoDefinition,
    GotoShow,
    FindReferences,
    Hover,
    LspRename,

    // Repeat
    DotRepeat,

    /// Repeat a command N times (count prefix).
    Repeat(usize, Box<Command>),

    // Completion
    CompletionNext,
    CompletionPrev,

    // No-op
    Noop,
}
