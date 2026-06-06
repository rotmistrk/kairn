//! Status bar construction — assembles the full status bar.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::status_bar::{Gravity, StatusBar, StatusSlot};
use txv_widgets::tiled_workspace::TiledWorkspace;
use txv_widgets::{BranchView, ClockView, ConfirmView, FocusGatedGroup, InputLine, MessageView, ModalKey};

use crate::commands::*;
use crate::settings::StatusKeys;
use crate::status_items::{CtxLangItem, CtxModeItem, CtxModifiedItem, CtxPositionItem, LspStatusItem};
use crate::views::todo_tree::TODO_STATUS_GROUP;

use super::helpers::{ALT_X, APPROX};
use super::keys;

/// Build the application status bar with all items configured.
pub fn build_status_bar(
    desktop: &TiledWorkspace,
    completer: Box<dyn Completer>,
    clock_interval: u16,
    root_dir: PathBuf,
    status_keys: &StatusKeys,
    clipboard: txv_core::clipboard_ring::ClipboardHandle,
) -> StatusBar {
    let mut bar = StatusBar::new();
    keys::add_workspace_bindings(&mut bar, desktop);
    keys::add_app_bindings(&mut bar, status_keys);
    keys::add_prefix_bindings(&mut bar);
    keys::add_dired_prefix(&mut bar);
    keys::add_tab_digit_bindings(&mut bar);
    add_command_items(&mut bar, completer, clipboard.clone());
    add_file_finder(&mut bar, root_dir.clone(), clipboard);
    add_todo_group(&mut bar);
    add_right_side(&mut bar, root_dir, clock_interval);
    bar
}

fn add_command_items(
    bar: &mut StatusBar,
    completer: Box<dyn Completer>,
    clipboard: txv_core::clipboard_ring::ClipboardHandle,
) {
    bar.add(StatusSlot::new(Box::new(ConfirmView::new(CM_CONFIRM, CM_CONFIRM_RESPONSE))).priority(10));
    let input = InputLine::new()
        .with_clipboard(clipboard.clone())
        .with_command(CM_EXECUTE_COMMAND)
        .with_completer(completer);
    let command_line = ModalKey::new("M-x", ":")
        .trigger_key(ALT_X)
        .trigger_key(APPROX)
        .prefill_command(CM_COMMAND_PREFILL)
        .terminal_command(CM_EXECUTE_COMMAND)
        .add_child(Box::new(input));
    bar.add(StatusSlot::new(Box::new(command_line)).priority(10).stretch(1));
}

fn add_file_finder(bar: &mut StatusBar, root: PathBuf, clipboard: txv_core::clipboard_ring::ClipboardHandle) {
    use crate::completer_file_finder::FileFinderCompleter;
    use crate::completer_symbol::SymbolFinderCompleter;

    let ctrl_p = KeyEvent {
        code: KeyCode::Char('p'),
        modifiers: KeyMod {
            ctrl: true,
            ..KeyMod::default()
        },
    };
    let input = InputLine::new()
        .with_clipboard(clipboard.clone())
        .with_command(CM_FILE_FINDER_OPEN)
        .with_completer(Box::new(FileFinderCompleter::new(root.clone())));
    let finder = ModalKey::new("", "file: ")
        .trigger_key(ctrl_p)
        .terminal_command(CM_FILE_FINDER_OPEN)
        .add_child(Box::new(input));
    bar.add(StatusSlot::new(Box::new(finder)).priority(5));

    let ctrl_t = KeyEvent {
        code: KeyCode::Char('t'),
        modifiers: KeyMod {
            ctrl: true,
            ..KeyMod::default()
        },
    };
    let sym_input = InputLine::new()
        .with_clipboard(clipboard.clone())
        .with_command(CM_FILE_FINDER_OPEN)
        .with_completer(Box::new(SymbolFinderCompleter::new(root)));
    let sym_finder = ModalKey::new("", "sym: ")
        .trigger_key(ctrl_t)
        .terminal_command(CM_FILE_FINDER_OPEN)
        .add_child(Box::new(sym_input));
    bar.add(StatusSlot::new(Box::new(sym_finder)).priority(5));
}
fn add_todo_group(bar: &mut StatusBar) {
    use txv_widgets::KeyLabelView;

    let key = |ch: char| KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod::default(),
    };
    let mut group = FocusGatedGroup::new(TODO_STATUS_GROUP);
    group.add_child(Box::new(KeyLabelView::new(key('i'), CM_TODO_TOGGLE_PROGRESS, "▶")));
    group.add_child(Box::new(KeyLabelView::new(key('\\'), CM_TODO_TOGGLE_PAUSE, "⏸")));
    group.add_child(Box::new(KeyLabelView::new(key('+'), CM_TODO_PRIORITY_UP, "+prio")));
    group.add_child(Box::new(KeyLabelView::new(key('='), CM_TODO_PRIORITY_UP, "")));
    group.add_child(Box::new(KeyLabelView::new(key('-'), CM_TODO_PRIORITY_DOWN, "")));
    group.add_child(Box::new(KeyLabelView::new(key('>'), CM_TODO_LOE_UP, ">loe")));
    group.add_child(Box::new(KeyLabelView::new(key('<'), CM_TODO_LOE_DOWN, "")));
    bar.add(StatusSlot::new(Box::new(group)).priority(8));
}

fn add_right_side(bar: &mut StatusBar, root_dir: PathBuf, clock_interval: u16) {
    bar.add(StatusSlot::new(Box::new(MessageView::new(5))).priority(10));
    bar.add(
        StatusSlot::new(Box::new(CtxModifiedItem::new()))
            .priority(7)
            .gravity(Gravity::Right),
    );
    bar.add(
        StatusSlot::new(Box::new(CtxPositionItem::new()))
            .priority(7)
            .gravity(Gravity::Right),
    );
    bar.add(
        StatusSlot::new(Box::new(CtxModeItem::new()))
            .priority(8)
            .gravity(Gravity::Right),
    );
    bar.add(
        StatusSlot::new(Box::new(CtxLangItem::new()))
            .priority(6)
            .gravity(Gravity::Right),
    );
    bar.add(
        StatusSlot::new(Box::new(LspStatusItem::new()))
            .priority(4)
            .gravity(Gravity::Right),
    );
    bar.add(
        StatusSlot::new(Box::new(BranchView::new(root_dir)))
            .priority(3)
            .gravity(Gravity::Right),
    );
    bar.add(
        StatusSlot::new(Box::new(ClockView::new(clock_interval)))
            .priority(4)
            .gravity(Gravity::Right),
    );
}
