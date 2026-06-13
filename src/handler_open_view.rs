//! View creation helpers for file-open handlers.

use std::fs;
use std::path::Path;

use txv_core::clipboard_ring::ClipboardHandle;
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
    ed.set_root_dir(state.roots().root_for(path).path().to_path_buf());
    let cl = state.command_list.clone();
    let dm = ed.delegate_mut();
    dm.command_list = cl;
    ed.editor_mut()
        .set_shared_state(state.shared_register.clone(), state.clipboard.clone());
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    ed.buffer_id = Some(state.buffers.register(Some(canon)));
    Box::new(ed)
}

/// Maximum file size (in bytes) for structured/table view. Larger files open as text.
const STRUCTURED_VIEW_MAX_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

/// Try to open as structured view (JSON/JSONC/JSONL/CSV/TSV).
/// Returns None (falls back to text editor) if file exceeds size limit.
pub(crate) fn try_open_structured(path: &Path, clipboard: Option<ClipboardHandle>) -> Option<Box<dyn View>> {
    let meta = fs::metadata(path).ok()?;
    if meta.len() > STRUCTURED_VIEW_MAX_SIZE {
        return None; // too large — fall back to text editor
    }
    let ext_os = path.extension()?;
    let ext = ext_os.to_str()?;
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
        "csv" | "tsv" | "tab" | "psv" => {
            let mut csv = CsvView::new(path, &content);
            if let Some(clip) = clipboard {
                csv.set_clipboard(clip);
            }
            Some(Box::new(csv))
        }
        _ => None,
    }
}

/// Open a file as an EditorView with optional goto/diff.
pub(crate) fn open_editor(path: &Path, state: &mut AppState, req: &OpenFileRequest) -> Box<dyn View> {
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut editor = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    editor.set_root_dir(state.roots().root_for(path).path().to_path_buf());
    editor
        .editor_mut()
        .set_shared_state(state.shared_register.clone(), state.clipboard.clone());
    if let Some(line) = req.line {
        editor.goto(line, req.col.unwrap_or(0));
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
    let mut csv = CsvView::new(&path, &content);
    csv.set_clipboard(state.clipboard.clone());
    let view: Box<dyn View> = Box::new(csv);
    try_insert_tab(d, state, sink, SlotId::Center, title, view);
}
