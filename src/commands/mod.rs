//! Kairn-specific command identifiers.
//! Core commands (CM_QUIT, CM_CLOSE, etc.) live in txv_core::commands.

mod context;
mod data;

pub use context::{ViewContext, ViewContextBuilder};
pub use data::{ConfirmContext, ContentChanged, DiffSplitRequest, OpenFileRequest, SplitRequest};

use txv_core::event::CommandId;

// File operations
pub const CM_APP_BASE: CommandId = txv_core::commands::CM_TXV_MAX + 1;
pub const CM_OPEN_FILE: CommandId = CM_APP_BASE;
pub const CM_SAVE: CommandId = CM_APP_BASE + 1;
pub const CM_NEW_SHELL: CommandId = CM_APP_BASE + 2;
pub const CM_NEW_KIRO: CommandId = CM_APP_BASE + 3;
pub const CM_FILE_DELETED: CommandId = CM_APP_BASE + 4;
pub const CM_FILE_CLOSED: CommandId = CM_APP_BASE + 5;
pub const CM_OPEN_FILE_FOCUS: CommandId = CM_APP_BASE + 6;

// Display
pub const CM_SHOW_HELP: CommandId = CM_APP_BASE + 30;
pub const CM_SHOW_MESSAGES: CommandId = CM_APP_BASE + 31;

// Tab close (app-level: notifies broker, handles save prompts)
pub const CM_TAB_CLOSE: CommandId = CM_APP_BASE + 17;

// Command mode
pub const CM_COMMAND_MODE: CommandId = CM_APP_BASE + 40;
pub const CM_EXECUTE_COMMAND: CommandId = CM_APP_BASE + 41;
pub const CM_SHELL_OUTPUT: CommandId = CM_APP_BASE + 42;
pub const CM_SET_GLOBAL: CommandId = CM_APP_BASE + 43;

// Editor status
pub const CM_MODE_CHANGED: CommandId = CM_APP_BASE + 50;
pub const CM_CURSOR_MOVED: CommandId = CM_APP_BASE + 51;
pub const CM_DIAGNOSTIC: CommandId = CM_APP_BASE + 52;
pub const CM_LSP_GOTO_DEF: CommandId = CM_APP_BASE + 53;
pub const CM_LSP_GOTO_SHOW: CommandId = CM_APP_BASE + 54;
pub const CM_LSP_FIND_REFS: CommandId = CM_APP_BASE + 55;
pub const CM_LSP_HOVER: CommandId = CM_APP_BASE + 56;
pub const CM_LSP_COMPLETION: CommandId = CM_APP_BASE + 57;
pub const CM_LSP_RENAME: CommandId = CM_APP_BASE + 58;
pub const CM_CODE_ACTION: CommandId = CM_APP_BASE + 59;
pub const CM_LSP_SIGNATURE_HELP: CommandId = CM_APP_BASE + 60;

// Build / clipboard
pub const CM_CLIPBOARD_PASTE: CommandId = CM_APP_BASE + 70;
pub const CM_BUILD: CommandId = CM_APP_BASE + 71;
pub const CM_RUN: CommandId = CM_APP_BASE + 72;
pub const CM_TEST: CommandId = CM_APP_BASE + 73;
pub const CM_TEST_FILE: CommandId = CM_APP_BASE + 74;
pub const CM_TEST_AT_CURSOR: CommandId = CM_APP_BASE + 75;
pub const CM_NEXT_ERROR: CommandId = CM_APP_BASE + 76;
pub const CM_PREV_ERROR: CommandId = CM_APP_BASE + 77;

pub const CM_SUSPEND: CommandId = CM_APP_BASE + 84;
pub const CM_PEEK: CommandId = CM_APP_BASE + 85;

// Git operations
pub const CM_GIT_STAGE: CommandId = CM_APP_BASE + 90;
pub const CM_GIT_UNSTAGE: CommandId = CM_APP_BASE + 91;
pub const CM_GIT_UNTRACK: CommandId = CM_APP_BASE + 92;
pub const CM_GIT_COMMIT: CommandId = CM_APP_BASE + 93;
pub const CM_GIT_COMMIT_PROMPT: CommandId = CM_APP_BASE + 94;
/// Activates the command input with pre-filled text.
pub const CM_COMMAND_PREFILL: CommandId = CM_APP_BASE + 95;
pub const CM_DIFF: CommandId = CM_APP_BASE + 96;
pub const CM_BLAME: CommandId = CM_APP_BASE + 97;
pub const CM_NOBLAME: CommandId = CM_APP_BASE + 98;

// LSP document sync
/// Editor content changed — triggers didChange to LSP server.
pub const CM_CONTENT_CHANGED: CommandId = CM_APP_BASE + 100;

/// Show results in a quickfix-style list (data: Vec<ResultEntry>).
pub const CM_SHOW_RESULTS: CommandId = CM_APP_BASE + 101;
pub const CM_GOTO_LINE: CommandId = CM_APP_BASE + 102;
pub const CM_GREP_RESULTS: CommandId = CM_APP_BASE + 103;
pub const CM_TOGGLE_THEME: CommandId = CM_APP_BASE + 104;
pub const CM_SET_SYNTAX_THEME: CommandId = CM_APP_BASE + 105;
pub const CM_SET_GLYPHS: CommandId = CM_APP_BASE + 106;

// Confirmation prompt (ConfirmItem in status bar)
pub const CM_CONFIRM: CommandId = CM_APP_BASE + 110;
pub const CM_CONFIRM_RESPONSE: CommandId = CM_APP_BASE + 111;
/// Sets the confirm context (data: ConfirmContext). Handled by main handler.
pub const CM_SET_CONFIRM_CONTEXT: CommandId = CM_APP_BASE + 112;

// Context broadcast
pub const CM_CONTEXT_UPDATE: CommandId = CM_APP_BASE + 120;

// Editor script operations (from Tcl bridge)
pub const CM_EDITOR_REPLACE_SELECTION: CommandId = CM_APP_BASE + 130;
pub const CM_EDITOR_DELETE_LINE: CommandId = CM_APP_BASE + 131;
pub const CM_EDITOR_REPLACE_WORD: CommandId = CM_APP_BASE + 132;
pub const CM_EDITOR_SEARCH: CommandId = CM_APP_BASE + 133;
pub const CM_EDITOR_CLEAR_HIGHLIGHT: CommandId = CM_APP_BASE + 134;
pub const CM_DIFF_REVERT: CommandId = CM_APP_BASE + 135;

// Hook triggers from editor
pub const CM_CHAR_INSERTED: CommandId = CM_APP_BASE + 140;
pub const CM_WORD_COMPLETED: CommandId = CM_APP_BASE + 141;

// Todo operations
pub const CM_TODO_NOTE_OPEN: CommandId = CM_APP_BASE + 150;
pub const CM_TODO_NOTE_SAVE: CommandId = CM_APP_BASE + 151;
/// Update Notes tab content (no focus change, no create if absent).
pub const CM_TODO_NOTE_UPDATE: CommandId = CM_APP_BASE + 152;

// Split view
pub const CM_SPLIT: CommandId = CM_APP_BASE + 160;
pub const CM_SPLIT_CLOSE: CommandId = CM_APP_BASE + 161;
pub const CM_OPEN_IN_SPLIT: CommandId = CM_APP_BASE + 162;
pub const CM_SPLIT_FOCUS: CommandId = CM_APP_BASE + 163;
pub const CM_DIFF_SPLIT: CommandId = CM_APP_BASE + 164;
pub const CM_LSP_STATUS_UPDATE: CommandId = CM_APP_BASE + 165;
pub const CM_SPLIT_LINKED: CommandId = CM_APP_BASE + 166;
pub const CM_GIT_LOG: CommandId = CM_APP_BASE + 167;
pub const CM_TODO_ACTION: CommandId = CM_APP_BASE + 168;

// Quit (app-level, checks unsaved before emitting CM_QUIT)
pub const CM_APP_QUIT: CommandId = CM_APP_BASE + 169;
pub const CM_SAVE_ALL: CommandId = CM_APP_BASE + 170;

// File tree dired operations
pub const CM_TREE_NEW_FILE: CommandId = CM_APP_BASE + 171;
pub const CM_TREE_NEW_DIR: CommandId = CM_APP_BASE + 172;
pub const CM_TREE_DELETE: CommandId = CM_APP_BASE + 173;
pub const CM_TREE_RENAME: CommandId = CM_APP_BASE + 174;
pub const CM_TREE_COPY: CommandId = CM_APP_BASE + 175;
pub const CM_TREE_MARK: CommandId = CM_APP_BASE + 176;
pub const CM_TREE_UNMARK_ALL: CommandId = CM_APP_BASE + 177;
pub const CM_TREE_MOVE_MARKED: CommandId = CM_APP_BASE + 178;
pub const CM_TREE_COPY_MARKED: CommandId = CM_APP_BASE + 179;
