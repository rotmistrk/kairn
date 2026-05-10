//! Status bar configuration — builds a composed StatusBar from items.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::status::StatusBar;
use txv_widgets::command_item::CommandItem;
use txv_widgets::status_indicators::{BranchItem, ModeItem, PositionItem};
use txv_widgets::status_items::{ClockItem, KeyLabelItem, MessageItem};

use crate::commands::*;

const ALT_X: KeyEvent = KeyEvent {
    code: KeyCode::Char('x'),
    modifiers: KeyMod {
        ctrl: false,
        alt: true,
        shift: false,
    },
};
const APPROX: KeyEvent = KeyEvent {
    code: KeyCode::Char('≈'),
    modifiers: KeyMod {
        ctrl: false,
        alt: false,
        shift: false,
    },
};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod::default(),
    }
}
fn ctrl(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    }
}
fn ctrl_shift(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: true,
            alt: false,
            shift: true,
        },
    }
}

/// Build the application status bar with all items configured.
pub fn build_status_bar(completer: Box<dyn Completer>, clock_interval: u16, root_dir: PathBuf) -> StatusBar {
    let mut bar = StatusBar::new();
    // Left key labels
    bar.add(KeyLabelItem::new(key(KeyCode::F(1)), CM_SHOW_HELP, "F1:Help"));
    bar.add(KeyLabelItem::new(key(KeyCode::F(2)), CM_FOCUS_LEFT, "F2:Tree"));
    bar.add(KeyLabelItem::new(key(KeyCode::F(3)), CM_FOCUS_CENTER, "F3:Main"));
    bar.add(KeyLabelItem::new(key(KeyCode::F(4)), CM_FOCUS_RIGHT, "F4:Term"));
    bar.add(KeyLabelItem::new(key(KeyCode::F(5)), CM_ZOOM_TOGGLE, "F5:Zoom"));
    bar.add(KeyLabelItem::new(key(KeyCode::F(6)), CM_SHOW_MESSAGES, "F6:Msg"));
    bar.add(KeyLabelItem::new(ctrl('q'), CM_QUIT, "^Q:Quit"));
    // Hidden hotkeys
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Left), CM_FOCUS_PREV));
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Right), CM_FOCUS_NEXT));
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Up), CM_TAB_DROPDOWN));
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Down), CM_TAB_DROPDOWN));
    // Panel resize: ≠ (Alt+=) grow, – (Alt+-) shrink
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('≠')), CM_PANEL_GROW));
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('–')), CM_PANEL_SHRINK));
    // Suspend and peek
    bar.add_active_only(KeyLabelItem::hidden(ctrl('z'), CM_SUSPEND));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('o'), CM_PEEK));
    // Command input (exclusive on activation)
    bar.add(
        CommandItem::new(&[ALT_X, APPROX], CM_EXECUTE_COMMAND)
            .with_label("M-x")
            .with_completer(completer),
    );
    // Right side
    bar.add(PositionItem::new(CM_CURSOR_MOVED));
    bar.add(ModeItem::new(CM_MODE_CHANGED));
    bar.add_visible_only(MessageItem::new(5));
    bar.add_visible_only(BranchItem::new(root_dir));
    bar.add_visible_only(ClockItem::new(clock_interval));
    bar
}
