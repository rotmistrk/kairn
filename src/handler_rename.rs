//! File rename handler — updates editor paths when files are moved on disk.

use txv_core::message::Message;
use txv_core::prelude::EventSink;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::views::editor::EditorView;

pub(crate) fn drain_renames(ctx: &mut CommandContext, state: &mut AppState) {
    let events = state.rename_watcher.as_ref().map(|w| w.drain()).unwrap_or_default();
    if events.is_empty() {
        return;
    }
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    for (old_path, new_path) in &events {
        apply_rename(desktop, old_path, new_path, &sink);
    }
}

fn apply_rename(
    desktop: &mut TiledWorkspace,
    old_path: &std::path::Path,
    new_path: &std::path::Path,
    sink: &EventSink,
) {
    let slot = SlotId::Center as usize;
    let Some(panel) = desktop.panel_mut(slot) else {
        return;
    };
    for idx in 0..panel.tab_count() {
        if !is_path_match(panel, idx, old_path) {
            continue;
        }
        update_editor_path(panel, idx, new_path);
        let msg = Message::info(
            "editor",
            format!("Renamed: {} → {}", old_path.display(), new_path.display()),
        );
        sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        break;
    }
}

fn is_path_match(panel: &mut txv_widgets::tab_panel::TabPanel, idx: usize, old_path: &std::path::Path) -> bool {
    panel
        .view_at_mut(idx)
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_ref::<EditorView>())
        .is_some_and(|ed| ed.path() == old_path)
}

fn update_editor_path(panel: &mut txv_widgets::tab_panel::TabPanel, idx: usize, new_path: &std::path::Path) {
    if let Some(ed) = panel
        .view_at_mut(idx)
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<EditorView>())
    {
        ed.set_path(new_path);
    }
    let title = new_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string();
    panel.set_title(idx, &title);
}
