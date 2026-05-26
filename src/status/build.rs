//! Status bar construction — assembles the full status bar.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::status_bar::{Gravity, StatusBar, StatusSlot};
use txv_widgets::tiled_workspace::TiledWorkspace;
use txv_widgets::{BranchView, ClockView, CommandLineView, ConfirmView, MessageView};

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
    keys::add_app_bindings(&mut bar, status_keys);
    keys::add_tab_digit_bindings(&mut bar);
    add_command_items(&mut bar, completer);
    add_right_side(&mut bar, root_dir, clock_interval);
    bar
}

fn add_command_items(bar: &mut StatusBar, completer: Box<dyn Completer>) {
    bar.add(StatusSlot::new(Box::new(ConfirmView::new(CM_CONFIRM, CM_CONFIRM_RESPONSE))).priority(10));
    bar.add(
        StatusSlot::new(Box::new(
            CommandLineView::new(&[ALT_X, APPROX], CM_EXECUTE_COMMAND)
                .with_label("M-x")
                .with_prefill_command(CM_COMMAND_PREFILL)
                .with_completer(completer),
        ))
        .priority(10),
    );
}

fn add_right_side(bar: &mut StatusBar, root_dir: PathBuf, clock_interval: u16) {
    bar.add(StatusSlot::new(Box::new(MessageView::new(5))).priority(9).stretch(1));
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
            .priority(2)
            .gravity(Gravity::Right),
    );
}
