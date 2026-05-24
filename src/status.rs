//! Status bar configuration — builds a composed StatusBar from items.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::status::StatusBar;
use txv_widgets::command_item::CommandItem;
use txv_widgets::confirm_item::ConfirmItem;
use txv_widgets::prefix_item::PrefixItem;
use txv_widgets::status_indicators::BranchItem;
use txv_widgets::status_items::{ClockItem, KeyLabelItem, MessageItem};
use txv_widgets::tiled_workspace::commands::CM_TW_ACTIVATE_TAB;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::commands::*;
use crate::settings::StatusKeys;
use crate::status_items::{CtxLangItem, CtxModeItem, CtxModifiedItem, CtxPositionItem, LspStatusItem};

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

/// Build the application status bar with all items configured.
pub fn build_status_bar(
    desktop: &TiledWorkspace,
    completer: Box<dyn Completer>,
    clock_interval: u16,
    root_dir: PathBuf,
    keys: &StatusKeys,
) -> StatusBar {
    let mut bar = StatusBar::new();
    add_workspace_bindings(&mut bar, desktop);
    add_prefix_bindings(&mut bar);
    add_app_bindings(&mut bar, keys);
    add_tab_digit_bindings(&mut bar);
    add_command_items(&mut bar, completer);
    add_right_side(&mut bar, root_dir, clock_interval);
    bar
}

/// Register TiledWorkspace's default key→command bindings.
fn add_workspace_bindings(bar: &mut StatusBar, desktop: &TiledWorkspace) {
    for (key, command, _payload) in desktop.default_bindings() {
        bar.add_active_only(KeyLabelItem::hidden(key, command));
    }
}

/// Ctrl-W prefix key sequence for subpanel management.
fn add_prefix_bindings(bar: &mut StatusBar) {
    use txv_widgets::tiled_workspace::commands::{
        CM_TW_CLOSE_OTHER_SUBPANEL, CM_TW_CLOSE_SUBPANEL, CM_TW_CYCLE_SUBPANEL, CM_TW_EQUALIZE_SUBPANEL,
        CM_TW_GROW_SUBPANEL, CM_TW_MOVE_TAB_SUBPANEL, CM_TW_SHRINK_SUBPANEL, CM_TW_SPLIT_H, CM_TW_SPLIT_V,
    };
    let prefix = PrefixItem::new(ctrl('w'), "C-w")
        .bind('s', CM_TW_SPLIT_H, "split")
        .bind('v', CM_TW_SPLIT_V, "vsplit")
        .bind('c', CM_TW_CLOSE_SUBPANEL, "close")
        .bind('o', CM_TW_CLOSE_OTHER_SUBPANEL, "only")
        .bind('w', CM_TW_CYCLE_SUBPANEL, "cycle")
        .bind('m', CM_TW_MOVE_TAB_SUBPANEL, "move")
        .bind('+', CM_TW_GROW_SUBPANEL, "grow")
        .bind('-', CM_TW_SHRINK_SUBPANEL, "shrink")
        .bind('=', CM_TW_EQUALIZE_SUBPANEL, "equal");
    bar.add(prefix);
}

/// App-specific bindings (not workspace navigation).
fn add_app_bindings(bar: &mut StatusBar, keys: &StatusKeys) {
    use txv_widgets::tiled_workspace::commands::{
        CM_TW_FOCUS_PANEL, CM_TW_GROW_H, CM_TW_GROW_SUBPANEL, CM_TW_GROW_V, CM_TW_SHRINK_H, CM_TW_SHRINK_SUBPANEL,
        CM_TW_SHRINK_V, CM_TW_ZOOM,
    };
    bar.add(KeyLabelItem::new(keys.help, CM_SHOW_HELP, "F1:Help"));
    bar.add(KeyLabelItem::new(keys.zoom, CM_TW_ZOOM, "F5:Zoom"));
    bar.add(KeyLabelItem::new(keys.messages, CM_SHOW_MESSAGES, "F6:Msg"));
    bar.add(KeyLabelItem::new(keys.quit, CM_QUIT, "^Q:Quit"));
    bar.add_active_only(KeyLabelItem::hidden_with_data(keys.tree, CM_TW_FOCUS_PANEL, 0));
    bar.add_active_only(KeyLabelItem::hidden_with_data(keys.main, CM_TW_FOCUS_PANEL, 1));
    bar.add_active_only(KeyLabelItem::hidden_with_data(keys.term, CM_TW_FOCUS_PANEL, 2));
    // macOS Option+=/- resize keys — grow/shrink subpanel
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('≠')), CM_TW_GROW_SUBPANEL));
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('–')), CM_TW_SHRINK_SUBPANEL));
    // Alt+Shift+Arrow resize keys
    let alt_shift = |code| KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: false,
            shift: true,
            alt: true,
        },
    };
    bar.add_active_only(KeyLabelItem::hidden(alt_shift(KeyCode::Right), CM_TW_GROW_H));
    bar.add_active_only(KeyLabelItem::hidden(alt_shift(KeyCode::Left), CM_TW_SHRINK_H));
    bar.add_active_only(KeyLabelItem::hidden(alt_shift(KeyCode::Down), CM_TW_GROW_V));
    bar.add_active_only(KeyLabelItem::hidden(alt_shift(KeyCode::Up), CM_TW_SHRINK_V));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('z'), CM_SUSPEND));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('o'), CM_PEEK));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('d'), CM_DIFF));
    bar.add_active_only(KeyLabelItem::hidden(ctrl('l'), CM_REPAINT));
    // Alt-\ layout cycle (macOS produces «)
    use txv_widgets::tiled_workspace::commands::CM_TW_LAYOUT_CYCLE;
    bar.add_active_only(KeyLabelItem::hidden(
        KeyEvent {
            code: KeyCode::Char('\\'),
            modifiers: KeyMod {
                ctrl: false,
                alt: true,
                shift: false,
            },
        },
        CM_TW_LAYOUT_CYCLE,
    ));
    bar.add_active_only(KeyLabelItem::hidden(key(KeyCode::Char('«')), CM_TW_LAYOUT_CYCLE));
}

/// Alt+digit and macOS Option+digit for tab switching.
fn add_tab_digit_bindings(bar: &mut StatusBar) {
    use txv_widgets::tiled_workspace::commands::CM_TW_TAB_DROPDOWN;
    let mac_digits = ['º', '¡', '™', '£', '¢', '∞', '§', '¶', '•', 'ª'];
    // Alt+0 → dropdown
    let alt_0 = KeyEvent {
        code: KeyCode::Char('0'),
        modifiers: KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    };
    bar.add_active_only(KeyLabelItem::hidden(alt_0, CM_TW_TAB_DROPDOWN));
    bar.add_active_only(KeyLabelItem::hidden(
        key(KeyCode::Char(mac_digits[0])),
        CM_TW_TAB_DROPDOWN,
    ));
    // Alt+1..9 → activate tab by label
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
        bar.add_active_only(KeyLabelItem::hidden_with_data(alt_key, CM_TW_ACTIVATE_TAB, tab_idx));
        let mac_key = key(KeyCode::Char(mac_digits[i as usize]));
        bar.add_active_only(KeyLabelItem::hidden_with_data(mac_key, CM_TW_ACTIVATE_TAB, tab_idx));
    }
}

fn add_command_items(bar: &mut StatusBar, completer: Box<dyn Completer>) {
    bar.add(ConfirmItem::new(CM_CONFIRM, CM_CONFIRM_RESPONSE));
    bar.add(
        CommandItem::new(&[ALT_X, APPROX], CM_EXECUTE_COMMAND)
            .with_label("M-x")
            .with_prefill_command(CM_COMMAND_PREFILL)
            .with_completer(completer),
    );
}

fn add_right_side(bar: &mut StatusBar, root_dir: PathBuf, clock_interval: u16) {
    bar.add(MessageItem::new(5));
    bar.add(CtxModifiedItem::new());
    bar.add(CtxPositionItem::new());
    bar.add(CtxModeItem::new());
    bar.add(CtxLangItem::new());
    bar.add(LspStatusItem::new());
    bar.add_visible_only(BranchItem::new(root_dir));
    bar.add_visible_only(ClockItem::new(clock_interval));
}
