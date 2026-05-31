//! View creation helpers for file-open handlers.

use std::fs;
use std::path::Path;

use txv_core::prelude::*;

use crate::commands::*;
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::structured::json_doc::JsonDoc;
use crate::structured::jsonl_doc::JsonlDoc;
use crate::views::csv_view::CsvView;
use crate::views::editor::EditorView;
use crate::views::struct_view::StructuredView;

pub(crate) fn open_editor_view(path: &Path, state: &mut AppState) -> Box<dyn View> {
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut ed = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    ed.set_root_dir(state.root_dir.clone());
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    ed.buffer_id = Some(state.buffers.register(Some(canon)));
    Box::new(ed)
}

/// Try to open as structured view (JSON/JSONC/JSONL/CSV/TSV).
pub(crate) fn try_open_structured(path: &Path) -> Option<Box<dyn View>> {
    let ext = path.extension()?.to_str()?;
    let content = fs::read_to_string(path).ok()?;
    match ext {
        "json" => {
            let doc = JsonDoc::parse(&content).ok()?;
            Some(Box::new(StructuredView::new(path, Box::new(doc))))
        }
        "jsonc" => {
            let doc = JsonDoc::parse_jsonc(&content).ok()?;
            Some(Box::new(StructuredView::new(path, Box::new(doc))))
        }
        "jsonl" | "ndjson" => {
            let doc = JsonlDoc::parse(&content).ok()?;
            Some(Box::new(StructuredView::new(path, Box::new(doc))))
        }
        "csv" | "tsv" | "tab" | "psv" => Some(Box::new(CsvView::new(path, &content))),
        _ => None,
    }
}

/// Open a file as an EditorView with optional goto/diff.
pub(crate) fn open_editor(path: &Path, state: &mut AppState, req: &OpenFileRequest) -> Box<dyn View> {
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut editor = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    editor.set_root_dir(state.root_dir.clone());
    if let (Some(line), Some(col)) = (req.line, req.col) {
        editor.goto(line, col);
    }
    if req.diff {
        editor.toggle_diff("");
    }
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let buf_id = state.buffers.register(Some(canon));
    editor.buffer_id = Some(buf_id);
    Box::new(editor)
}

/// Open the current file as a CSV table view.
pub(crate) fn open_as_csv(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState) {
    use crate::views::editor::EditorView;
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    let abs_path = d
        .panel_mut(SlotId::Center as usize)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_ref::<EditorView>())
        .map(|ev| ev.path().to_path_buf());
    let Some(path) = abs_path else {
        return;
    };
    if !path.is_file() {
        return;
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return;
    };
    let abs_key = path.to_string_lossy().to_string();
    let title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string();
    let panel = d.panel_mut(SlotId::Center as usize);
    if let Some(p) = panel {
        p.close_active();
    }
    state.broker.close(&abs_key);
    let _ = state.broker.open(&abs_key, SlotId::Center, 0);
    let view: Box<dyn View> = Box::new(CsvView::new(&path, &content));
    try_insert_tab(d, state, sink, SlotId::Center, title, view);
}
