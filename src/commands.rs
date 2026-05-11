//! Kairn-specific command identifiers.
//! Core commands (CM_QUIT, CM_CLOSE, etc.) live in txv_core::commands.

use txv_core::event::CommandId;

// File operations
pub const CM_OPEN_FILE: CommandId = 100;
pub const CM_OPEN_FILE_FOCUS: CommandId = 106;
pub const CM_SAVE: CommandId = 101;
pub const CM_NEW_SHELL: CommandId = 102;
pub const CM_NEW_KIRO: CommandId = 103;
pub const CM_FILE_DELETED: CommandId = 104;
pub const CM_FILE_CLOSED: CommandId = 105;

// Focus / slot navigation
pub const CM_FOCUS_LEFT: CommandId = 110;
pub const CM_FOCUS_CENTER: CommandId = 111;
pub const CM_FOCUS_RIGHT: CommandId = 112;
pub const CM_FOCUS_BOTTOM: CommandId = 113;
pub const CM_ZOOM_TOGGLE: CommandId = 114;
pub const CM_TAB_NEXT: CommandId = 115;
pub const CM_TAB_PREV: CommandId = 116;
pub const CM_TAB_CLOSE: CommandId = 117;
pub const CM_FOCUS_TAB: CommandId = 118;
pub const CM_TAB_DROPDOWN: CommandId = 119;

// Display
pub const CM_SHOW_HELP: CommandId = 120;
pub const CM_SHOW_MESSAGES: CommandId = 121;

// Command mode
pub const CM_COMMAND_MODE: CommandId = 130;
pub const CM_EXECUTE_COMMAND: CommandId = 131;
pub const CM_SHELL_OUTPUT: CommandId = 132;
pub const CM_SET_GLOBAL: CommandId = 133;

// Editor status
pub const CM_MODE_CHANGED: CommandId = 141;
pub const CM_CURSOR_MOVED: CommandId = 142;
pub const CM_DIAGNOSTIC: CommandId = 143;
pub const CM_LSP_GOTO_DEF: CommandId = 144;
pub const CM_LSP_FIND_REFS: CommandId = 145;
pub const CM_LSP_HOVER: CommandId = 146;
pub const CM_LSP_COMPLETION: CommandId = 147;

// Clipboard
pub const CM_CLIPBOARD_PASTE: CommandId = 150;
pub const CM_BUILD: CommandId = 151;
pub const CM_RUN: CommandId = 152;
pub const CM_TEST: CommandId = 153;
pub const CM_TEST_FILE: CommandId = 154;
pub const CM_TEST_AT_CURSOR: CommandId = 155;
pub const CM_NEXT_ERROR: CommandId = 156;
pub const CM_PREV_ERROR: CommandId = 157;

// Panel resize
pub const CM_PANEL_GROW: CommandId = 160;
pub const CM_PANEL_SHRINK: CommandId = 161;
pub const CM_PANEL_GROW_V: CommandId = 162;
pub const CM_PANEL_SHRINK_V: CommandId = 163;
pub const CM_SUSPEND: CommandId = 164;
pub const CM_PEEK: CommandId = 165;
