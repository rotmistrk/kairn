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
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,
    HalfPageUp,
    HalfPageDown,

    // Editing
    InsertChar(char),
    InsertNewline,
    DeleteCharForward,
    DeleteCharBackward,
    DeleteLine,
    DeleteWord,
    NewlineBelow,
    NewlineAbove,

    // Undo/redo
    Undo,
    Redo,

    // Clipboard
    YankLine,
    Paste,

    // Mode
    EnterInsertMode,
    EnterInsertAfter,
    EnterInsertLineEnd,
    EnterInsertBelow,
    EnterInsertAbove,
    ExitInsertMode,

    // Ex
    ExCommand(String),

    // File
    Save,
    CloseBuffer,

    // No-op
    Noop,
}
