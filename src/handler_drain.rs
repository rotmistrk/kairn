//! Background task drain — polls grep and build tasks for results.

use txv_core::program::CommandContext;

use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;

/// Drain grep results from background thread into the ResultsView.
pub fn drain_grep(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((title, gs, root)) = state.grep_pending.take() else {
        return;
    };
    if let Some(err) = gs.take_error() {
        let msg = txv_core::message::Message::new(txv_core::message::MsgLevel::Error, "grep", err);
        ctx.queue
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    let entries = gs.take_entries();
    let done = gs.is_done();
    if !entries.is_empty() || done {
        if let Some(desktop) = downcast_desktop(ctx.desktop) {
            if let Some(view) = desktop.active_view_mut(SlotId::Right) {
                if let Some(rv) = view
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<crate::views::results::ResultsView>())
                {
                    rv.append(entries, done);
                }
            }
        }
    }
    if !done {
        state.grep_pending = Some((title, gs, root));
    }
}

/// Drain build/test results from background thread into the ResultsView.
pub fn drain_build(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((title, task, root)) = state.build_pending.take() else {
        return;
    };
    if let Some(err) = task.take_error() {
        let msg = txv_core::message::Message::new(txv_core::message::MsgLevel::Error, "build", err);
        ctx.queue
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    let entries = task.take_entries();
    let done = task.is_done();
    if !entries.is_empty() || done {
        for e in &entries {
            if !e.path.as_os_str().is_empty() {
                state.build_errors.push(crate::build::ErrorLocation {
                    file: e
                        .path
                        .strip_prefix(&root)
                        .unwrap_or(&e.path)
                        .to_string_lossy()
                        .to_string(),
                    line: e.line + 1,
                    col: e.col + 1,
                    message: e.text.clone(),
                });
            }
        }
        if let Some(desktop) = downcast_desktop(ctx.desktop) {
            if let Some(view) = desktop.active_view_mut(SlotId::Right) {
                if let Some(rv) = view
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<crate::views::results::ResultsView>())
                {
                    rv.append(entries, done);
                }
            }
        }
    }
    if !done {
        state.build_pending = Some((title, task, root));
    }
}
