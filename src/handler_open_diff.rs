//! Diff and view-mode toggling extracted from handler_open.

use txv_core::prelude::*;

use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::handler_open_view::{open_editor_view, try_open_structured};
use crate::views::csv_view::CsvView;
use crate::views::editor::{EditorView, EditorViewDiffExt};
use crate::views::struct_view::StructuredView;
use txv_widgets::tiled_workspace::TiledWorkspace;

pub(crate) use crate::handler_open_view::open_as_csv;

pub(crate) fn activate_diff_on_focused(desktop: &mut TiledWorkspace, diff_base: Option<&str>) {
    let slot = SlotId::Center as usize;
    let editor = desktop
        .panel_mut(slot)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<EditorView>());
    if let Some(ed) = editor {
        let args = diff_base.unwrap_or("");
        ed.toggle_diff(args);
        ed.flush_pending();
    }
}

/// Initial tab label — just the filename.
fn initial_title(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string()
}

/// Toggle the active center tab between structured and text view.
pub(crate) fn toggle_view_mode(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState, to_structured: bool) {
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    let abs_path = d
        .panel_mut(SlotId::Center as usize)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| {
            a.downcast_ref::<EditorView>()
                .map(|ev| ev.path().to_path_buf())
                .or_else(|| a.downcast_ref::<StructuredView>().map(|sv| sv.path.clone()))
                .or_else(|| a.downcast_ref::<CsvView>().map(|cv| cv.path().to_path_buf()))
        });
    let Some(path) = abs_path else {
        return;
    };
    if !path.is_file() {
        return;
    }
    let abs_key = path.to_string_lossy().to_string();
    let title = initial_title(&path);
    let panel = d.panel_mut(SlotId::Center as usize);
    if let Some(p) = panel {
        p.close_active();
    }
    state.workspace_mut().broker_mut().close(&abs_key);
    let _ = state.workspace_mut().broker_mut().open(&abs_key, SlotId::Center, 0);
    let view: Box<dyn View> = if to_structured {
        try_open_structured(&path, Some(state.editor().clipboard().clone()))
            .unwrap_or_else(|| open_editor_view(&path, state))
    } else {
        open_editor_view(&path, state)
    };
    try_insert_tab(d, state, sink, SlotId::Center, title, view);
}
