//! Status bar construction — assembles the full status bar.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::status_bar::{Gravity, StatusBar, StatusSlot};
use txv_widgets::tiled_workspace::TiledWorkspace;
use txv_widgets::{BranchView, ClockView, ConfirmView, InputLine, MessageView, ModalKey};

use crate::commands::*;
use crate::settings::StatusKeys;
use crate::status_items::{CtxLangItem, CtxModeItem, CtxModifiedItem, CtxPositionItem, LspStatusItem};

use super::helpers::{ALT_X, APPROX};
use super::keys;

/// Build the application status bar with all items configured.
pub fn build_status_bar(
    desktop: &TiledWorkspace,
    completer: Box<dyn Completer>,
    clock_interval: u16,
    root_dir: PathBuf,
    status_keys: &StatusKeys,
) -> StatusBar {
    let mut bar = StatusBar::new();
    keys::add_workspace_bindings(&mut bar, desktop);
    keys::add_prefix_bindings(&mut bar);
    keys::add_dired_prefix(&mut bar);
    keys::add_app_bindings(&mut bar, status_keys);
    keys::add_tab_digit_bindings(&mut bar);
    add_command_items(&mut bar, completer);
    add_right_side(&mut bar, root_dir, clock_interval);
    bar
}

fn add_command_items(bar: &mut StatusBar, completer: Box<dyn Completer>) {
    bar.add(StatusSlot::new(Box::new(ConfirmView::new(CM_CONFIRM, CM_CONFIRM_RESPONSE))).priority(10));
    let input = InputLine::new()
        .with_command(CM_EXECUTE_COMMAND)
        .with_completer(completer);
    let command_line = ModalKey::new("M-x", ":")
        .trigger_key(ALT_X)
        .trigger_key(APPROX)
        .prefill_command(CM_COMMAND_PREFILL)
        .add_child(Box::new(input));
    bar.add(StatusSlot::new(Box::new(command_line)).priority(10).stretch(1));
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
