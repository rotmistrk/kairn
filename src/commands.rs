//! Kairn-specific command identifiers.
//! Core commands (CM_QUIT, CM_CLOSE, etc.) live in txv_core::commands.

use txv_core::event::CommandId;

// File operations
pub const CM_OPEN_FILE: CommandId = 100;
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
