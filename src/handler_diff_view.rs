//! Handlers for DiffView lifecycle: open, exit, revert.

use std::path::PathBuf;

use txv_core::message::Message;
use txv_core::program::CommandContext;

use crate::desktop::SlotId;
use crate::handler::downcast_desktop;
use crate::views::diff_view::DiffView;
use crate::views::editor::diff_model::DiffState;
use crate::views::editor::EditorView;

pub(crate) fn handle_diff_open_view(ctx: &mut CommandContext) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((ds, content, path, show_numbers)) = boxed.downcast_ref::<(DiffState, String, PathBuf, bool)>() else {
        return;
    };
    let ds = ds.clone();
    let content = content.clone();
    let path = path.clone();
    let show_numbers = *show_numbers;
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    let title = format!("[diff] {}", path.file_name().unwrap_or_default().to_string_lossy());
    let diff_view = DiffView::new(ds, &content, path, show_numbers);
    panel.insert_tab(title, Box::new(diff_view));
    let new_idx = panel.tab_count() - 1;
    panel.set_active(new_idx);
}

pub(crate) fn handle_diff_exit(ctx: &mut CommandContext) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    let idx = panel.active_index();
    panel.remove_tab(idx);
}

pub(crate) fn handle_diff_revert(ctx: &mut CommandContext) {
    let cursor = ctx
        .data()
        .as_ref()
        .and_then(|b| b.downcast_ref::<usize>().copied())
        .unwrap_or(0);
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    let Some(view) = panel.active_view_mut() else {
        return;
    };
    let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    if let Some(ds) = ev.delegate_mut().diff_state_mut() {
        ds.cursor = cursor;
    }
    match ev.revert_hunk() {
        Ok(msg) => {
            let m = Message::info("editor", msg);
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(m)));
        }
        Err(e) => {
            let m = Message::error("editor", e);
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(m)));
        }
    }
}
