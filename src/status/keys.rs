//! Key binding registration for the status bar.

use txv_core::prelude::*;
use txv_core::status_bar::{StatusBar, StatusSlot};
use txv_widgets::tiled_workspace::commands::CM_TW_ACTIVATE_TAB;
use txv_widgets::tiled_workspace::TiledWorkspace;
use txv_widgets::{FocusGatedGroup, KeyLabelView, ModalKey};

use crate::commands::*;
use crate::settings::StatusKeys;
use crate::views::tree::DIRED_STATUS_GROUP;

use super::helpers::{alt, ctrl, key};

/// Register TiledWorkspace's default key→command bindings (hidden).
pub fn add_workspace_bindings(bar: &mut StatusBar, desktop: &TiledWorkspace) {
    for (k, command, _payload) in desktop.default_bindings() {
        bar.add(StatusSlot::new(Box::new(KeyLabelView::new(k, command, ""))));
    }
}

/// Alt-f prefix key sequence for file tree dired operations.
pub fn add_dired_prefix(bar: &mut StatusBar) {
    let b = |ch: char, cmd, label| -> Box<dyn View> { Box::new(KeyLabelView::new(key(KeyCode::Char(ch)), cmd, label)) };
    let prefix = ModalKey::new("M-f", "File: ")
        .trigger_key(alt('f'))
        .trigger_key(key(KeyCode::Char('ƒ')))
        .cancel_on_miss()
        .add_child(b('n', CM_TREE_NEW_FILE, "new"))
        .add_child(b('N', CM_TREE_NEW_DIR, "dir"))
        .add_child(b('d', CM_TREE_DELETE, "del"))
        .add_child(b('r', CM_TREE_RENAME, "ren"))
        .add_child(b('c', CM_TREE_COPY, "copy"))
        .add_child(b('m', CM_TREE_MARK, "mark"))
        .add_child(b('u', CM_TREE_UNMARK_ALL, "unmark"))
        .add_child(b('M', CM_TREE_MOVE_MARKED, "Move"))
        .add_child(b('C', CM_TREE_COPY_MARKED, "Copy"));
    let mut group = FocusGatedGroup::new(DIRED_STATUS_GROUP);
    group.add_child(Box::new(prefix));
    bar.add(StatusSlot::new(Box::new(group)).priority(8).stretch(1));
}

/// Ctrl-W prefix key sequence for subpanel management.
pub fn add_prefix_bindings(bar: &mut StatusBar) {
    use txv_widgets::tiled_workspace::commands::{
        CM_TW_CLOSE_OTHER_SUBPANEL, CM_TW_CLOSE_SUBPANEL, CM_TW_CYCLE_SUBPANEL, CM_TW_EQUALIZE_SUBPANEL,
        CM_TW_GROW_SUBPANEL, CM_TW_MOVE_TAB_SUBPANEL, CM_TW_SHRINK_SUBPANEL, CM_TW_SPLIT_H, CM_TW_SPLIT_V,
    };
    let b = |ch: char, cmd, label| -> Box<dyn View> { Box::new(KeyLabelView::new(key(KeyCode::Char(ch)), cmd, label)) };
    let prefix = ModalKey::new("C-w", "C-w: ")
        .trigger_key(ctrl('w'))
        .cancel_on_miss()
        .add_child(b('s', CM_TW_SPLIT_H, "split"))
        .add_child(b('v', CM_TW_SPLIT_V, "vsplit"))
        .add_child(b('c', CM_TW_CLOSE_SUBPANEL, "close"))
        .add_child(b('o', CM_TW_CLOSE_OTHER_SUBPANEL, "only"))
        .add_child(b('w', CM_TW_CYCLE_SUBPANEL, "cycle"))
        .add_child(b('m', CM_TW_MOVE_TAB_SUBPANEL, "move"))
        .add_child(b('+', CM_TW_GROW_SUBPANEL, "grow"))
        .add_child(b('-', CM_TW_SHRINK_SUBPANEL, "shrink"))
        .add_child(b('=', CM_TW_EQUALIZE_SUBPANEL, "equal"));
    bar.add(StatusSlot::new(Box::new(prefix)).priority(6));
}

/// Visible app bindings (shown in status bar).
pub fn add_app_bindings(bar: &mut StatusBar, keys: &StatusKeys) {
    use txv_widgets::tiled_workspace::commands::{CM_TW_FOCUS_PANEL, CM_TW_ZOOM};
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(keys.help, CM_SHOW_HELP, "~F1~:Help"))).priority(6));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(keys.zoom, CM_TW_ZOOM, "~F5~:Zoom"))).priority(5));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(keys.messages, CM_SHOW_MESSAGES, "~F6~:Msg"))).priority(5));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(keys.quit, CM_APP_QUIT, "~C-q~:Quit"))).priority(9));
    bar.add(StatusSlot::new(Box::new(
        KeyLabelView::new(keys.tree, CM_TW_FOCUS_PANEL, "").with_data(0),
    )));
    bar.add(StatusSlot::new(Box::new(
        KeyLabelView::new(keys.main, CM_TW_FOCUS_PANEL, "").with_data(1),
    )));
    bar.add(StatusSlot::new(Box::new(
        KeyLabelView::new(keys.term, CM_TW_FOCUS_PANEL, "").with_data(2),
    )));
    add_hidden_bindings(bar);
}

/// Hidden bindings (resize, layout, suspend — no visible label).
fn add_hidden_bindings(bar: &mut StatusBar) {
    add_hidden_resize(bar);
    add_hidden_misc(bar);
}

fn add_hidden_resize(bar: &mut StatusBar) {
    use txv_widgets::tiled_workspace::commands::{
        CM_TW_GROW_H, CM_TW_GROW_SUBPANEL, CM_TW_GROW_V, CM_TW_SHRINK_H, CM_TW_SHRINK_SUBPANEL, CM_TW_SHRINK_V,
    };
    let h = |k: KeyEvent, cmd| StatusSlot::new(Box::new(KeyLabelView::new(k, cmd, "")));
    bar.add(h(key(KeyCode::Char('≠')), CM_TW_GROW_SUBPANEL));
    bar.add(h(key(KeyCode::Char('–')), CM_TW_SHRINK_SUBPANEL));
    let alt_shift = |code| KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: false,
            shift: true,
            alt: true,
        },
    };
    bar.add(h(alt_shift(KeyCode::Right), CM_TW_GROW_H));
    bar.add(h(alt_shift(KeyCode::Left), CM_TW_SHRINK_H));
    bar.add(h(alt_shift(KeyCode::Down), CM_TW_GROW_V));
    bar.add(h(alt_shift(KeyCode::Up), CM_TW_SHRINK_V));
}

fn add_hidden_misc(bar: &mut StatusBar) {
    use txv_widgets::tiled_workspace::commands::{CM_TW_LAYOUT_CYCLE, CM_TW_TAB_CLOSE};
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(
        alt('w'),
        CM_TW_TAB_CLOSE,
        "",
    ))));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(ctrl('z'), CM_SUSPEND, ""))));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(ctrl('o'), CM_PEEK, ""))));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(ctrl('d'), CM_DIFF, ""))));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(ctrl('l'), CM_REPAINT, ""))));
    let alt_backslash = KeyEvent {
        code: KeyCode::Char('\\'),
        modifiers: KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    };
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(
        alt_backslash,
        CM_TW_LAYOUT_CYCLE,
        "",
    ))));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(
        key(KeyCode::Char('«')),
        CM_TW_LAYOUT_CYCLE,
        "",
    ))));
}

/// Alt+digit and macOS Option+digit for tab switching.
pub fn add_tab_digit_bindings(bar: &mut StatusBar) {
    use txv_widgets::tiled_workspace::commands::CM_TW_TAB_DROPDOWN;
    let mac_digits = ['º', '¡', '™', '£', '¢', '∞', '§', '¶', '•', 'ª'];
    let alt_0 = KeyEvent {
        code: KeyCode::Char('0'),
        modifiers: KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    };
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(
        alt_0,
        CM_TW_TAB_DROPDOWN,
        "",
    ))));
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(
        key(KeyCode::Char(mac_digits[0])),
        CM_TW_TAB_DROPDOWN,
        "",
    ))));
    for i in 1..10u8 {
        let tab_idx = (i - 1) as u16;
        let alt_key = KeyEvent {
            code: KeyCode::Char((b'0' + i) as char),
            modifiers: KeyMod {
                ctrl: false,
                alt: true,
                shift: false,
            },
        };
        bar.add(StatusSlot::new(Box::new(
            KeyLabelView::new(alt_key, CM_TW_ACTIVATE_TAB, "").with_data(tab_idx),
        )));
        let mac_key = key(KeyCode::Char(mac_digits[i as usize]));
        bar.add(StatusSlot::new(Box::new(
            KeyLabelView::new(mac_key, CM_TW_ACTIVATE_TAB, "").with_data(tab_idx),
        )));
    }
}
