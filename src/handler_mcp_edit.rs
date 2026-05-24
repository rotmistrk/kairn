//! MCP edit operations — buffer edits, cursor, save, diagnostics.

use crate::desktop::SlotId;
use crate::views::editor::EditorView;
use txv_widgets::tiled_workspace::TiledWorkspace;

/// Find an editor view by tab name in the center panel.
pub(crate) fn find_editor<'a>(desktop: &'a mut TiledWorkspace, name: &str) -> Result<&'a mut EditorView, String> {
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return Err("Panel not found".to_string());
    };
    for i in 0..panel.tab_count() {
        if panel.tab_title(i) == Some(name) {
            let view = panel.view_at_mut(i).ok_or("View not accessible")?;
            let any = view.as_any_mut().ok_or("View has no Any")?;
            return any
                .downcast_mut::<EditorView>()
                .ok_or_else(|| format!("Tab '{name}' is not an editor"));
        }
    }
    Err(format!("Tab not found: {name}"))
}

pub(crate) fn mcp_edit_buffer(
    desktop: &mut TiledWorkspace,
    name: &str,
    start_line: usize,
    end_line: usize,
    text: &str,
) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    let mut buf = editor.editor.buf();
    let line_count = buf.line_count();
    let start = start_line.min(line_count);
    let end = end_line.min(line_count);
    if start > end {
        return Err("start_line > end_line".to_string());
    }
    let start_offset = buf.line_col_to_offset(start, 0).unwrap_or(0);
    let end_offset = if end >= line_count {
        buf.content().len()
    } else {
        buf.line_col_to_offset(end, 0).unwrap_or(buf.content().len())
    };
    if end_offset > start_offset {
        buf.delete(start_offset, end_offset);
    }
    if !text.is_empty() {
        let insert_at = buf.line_col_to_offset(start, 0).unwrap_or(0);
        buf.insert(insert_at, text);
    }
    Ok(serde_json::json!({"edited": name}))
}

pub(crate) fn mcp_insert_text(
    desktop: &mut TiledWorkspace,
    name: &str,
    line: usize,
    col: usize,
    text: &str,
) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    editor.editor.buf().insert_at(line, col, text);
    Ok(serde_json::json!({"inserted": text.len()}))
}

pub(crate) fn mcp_set_cursor(
    desktop: &mut TiledWorkspace,
    name: &str,
    line: usize,
    col: usize,
) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    editor.goto(line as u32, col as u32);
    Ok(serde_json::json!({"cursor": {"line": line, "col": col}}))
}

pub(crate) fn mcp_save_file(desktop: &mut TiledWorkspace, name: &str) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    editor.save().map_err(|e| format!("Save failed: {e}"))?;
    Ok(serde_json::json!({"saved": name}))
}

pub(crate) fn mcp_get_diagnostics(desktop: &mut TiledWorkspace, name: &str) -> Result<serde_json::Value, String> {
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return Err("Panel not found".to_string());
    };
    let mut all_diags = Vec::new();
    for i in 0..panel.tab_count() {
        let title = panel.tab_title(i).unwrap_or_default().to_string();
        if !name.is_empty() && title != name {
            continue;
        }
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(editor) = any.downcast_ref::<EditorView>() else {
            continue;
        };
        if let Some(diags) = &editor.diagnostics {
            for d in diags {
                let severity = match d.severity {
                    crate::lsp::diagnostics::Severity::Error => "error",
                    crate::lsp::diagnostics::Severity::Warning => "warning",
                    crate::lsp::diagnostics::Severity::Info => "info",
                    crate::lsp::diagnostics::Severity::Hint => "hint",
                };
                all_diags.push(serde_json::json!({
                    "file": title,
                    "line": d.line,
                    "col": d.col_start,
                    "severity": severity,
                    "message": d.message,
                }));
            }
        }
    }
    Ok(serde_json::json!({"diagnostics": all_diags}))
}
