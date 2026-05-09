//! Well-known command identifiers for kairn.
//!
//! Views communicate via commands — never by knowing about siblings.
//! These constants define the shared vocabulary.

use txv_widgets::CommandId;

// Application lifecycle
pub const CM_QUIT: CommandId = 1;
pub const CM_OPEN_FILE: CommandId = 2;
pub const CM_SAVE: CommandId = 3;
pub const CM_CLOSE: CommandId = 4;

// Shell/Kiro tab creation
pub const CM_NEW_SHELL: CommandId = 20;
pub const CM_NEW_KIRO: CommandId = 21;

// Focus navigation between slots
pub const CM_FOCUS_LEFT: CommandId = 30;
pub const CM_FOCUS_CENTER: CommandId = 31;
pub const CM_FOCUS_RIGHT: CommandId = 32;
pub const CM_FOCUS_BOTTOM: CommandId = 33;
pub const CM_FOCUS_NEXT_SLOT: CommandId = 34;

// Tab management within a slot
pub const CM_TAB_NEXT: CommandId = 40;
pub const CM_TAB_PREV: CommandId = 41;
pub const CM_TAB_CLOSE: CommandId = 42;

// Slot resize and zoom
pub const CM_SLOT_GROW: CommandId = 50;
pub const CM_SLOT_SHRINK: CommandId = 51;
pub const CM_ZOOM_TOGGLE: CommandId = 52;

// UI toggles
pub const CM_SHOW_HELP: CommandId = 60;
pub const CM_TOGGLE_LEFT: CommandId = 61;
pub const CM_TOGGLE_RIGHT: CommandId = 62;
pub const CM_TOGGLE_BOTTOM: CommandId = 63;
pub const CM_CYCLE_LAYOUT: CommandId = 64;

// Internal: insert a view into a slot (carries payload via InsertRequest)
pub const CM_INSERT_VIEW: CommandId = 100;
