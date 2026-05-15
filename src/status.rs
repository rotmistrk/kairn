//! Status bar configuration — builds a composed StatusBar from items.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::status::StatusBar;
use txv_widgets::command_item::CommandItem;
use txv_widgets::confirm_item::ConfirmItem;
use txv_widgets::status_indicators::BranchItem;
use txv_widgets::status_items::{ClockItem, KeyLabelItem, MessageItem};

use crate::commands::*;
use crate::settings::StatusKeys;
use crate::status_items::{CtxLangItem, CtxModeItem, CtxModifiedItem, CtxPositionItem};

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
pub fn build_status_bar(
    completer: Box<dyn Completer>,
    clock_interval: u16,
    root_dir: PathBuf,
    keys: &StatusKeys,
) -> StatusBar {
    let mut bar = StatusBar::new();
    // Left key labels (from config)
    bar.add(KeyLabelItem::new(keys.help, CM_SHOW_HELP, "F1:Help"));
    bar.add(KeyLabelItem::new(keys.tree, CM_FOCUS_LEFT, "F2:Tree"));
    bar.add(KeyLabelItem::new(keys.main, CM_FOCUS_CENTER, "F3:Main"));
    bar.add(KeyLabelItem::new(keys.term, CM_FOCUS_RIGHT, "F4:Term"));
    bar.add(KeyLabelItem::new(keys.zoom, CM_ZOOM_TOGGLE, "F5:Zoom"));
    bar.add(KeyLabelItem::new(keys.messages, CM_SHOW_MESSAGES, "F6:Msg"));
    bar.add(KeyLabelItem::new(keys.quit, CM_QUIT, "^Q:Quit"));
    // Hidden hotkeys
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Left), CM_FOCUS_PREV));
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Right), CM_FOCUS_NEXT));
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Up), CM_TAB_DROPDOWN_UP));
    bar.add_active_only(KeyLabelItem::hidden(ctrl_shift(KeyCode::Down), CM_TAB_DROPDOWN_DOWN));
    // Panel resize: ≠ (Alt+=) grow, – (Alt+-) shrink, ± (Alt+Shift+=) grow vertical, — (Alt+Shift+-) shrink vertical
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('≠')), CM_PANEL_GROW));
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('–')), CM_PANEL_SHRINK));
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('±')), CM_PANEL_GROW_V));
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('—')), CM_PANEL_SHRINK_V));
    // Suspend and peek
    bar.add_active_only(KeyLabelItem::hidden(ctrl('z'), CM_SUSPEND));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('o'), CM_PEEK));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('d'), CM_DIFF));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('l'), CM_REPAINT));
    // Alt-0..9: select tab by number (Linux: alt+digit, macOS: Unicode chars)
    let mac_digits = ['º', '¡', '™', '£', '¢', '∞', '§', '¶', '•', 'ª'];
    for i in 0..10u8 {
        let alt_key = KeyEvent {
            code: KeyCode::Char((b'0' + i) as char),
            modifiers: KeyMod {
                ctrl: false,
                alt: true,
                shift: false,
            },
        };
        bar.add_active_only(KeyLabelItem::hidden_with_data(alt_key, CM_FOCUS_TAB, i as u16));
        let mac_key = key(KeyCode::Char(mac_digits[i as usize]));
        bar.add_active_only(KeyLabelItem::hidden_with_data(mac_key, CM_FOCUS_TAB, i as u16));
    }
    // Command input (exclusive on activation)
    bar.add(ConfirmItem::new(CM_CONFIRM, CM_CONFIRM_RESPONSE));
    bar.add(
        CommandItem::new(&[ALT_X, APPROX], CM_EXECUTE_COMMAND)
            .with_label("M-x")
            .with_prefill_command(CM_COMMAND_PREFILL)
            .with_completer(completer),
    );
    // Right side
    bar.add(MessageItem::new(5));
    bar.add(CtxModifiedItem::new());
    bar.add(CtxPositionItem::new());
    bar.add(CtxModeItem::new());
    bar.add(CtxLangItem::new());
    bar.add_visible_only(BranchItem::new(root_dir));
    bar.add_visible_only(ClockItem::new(clock_interval));
    bar
}
