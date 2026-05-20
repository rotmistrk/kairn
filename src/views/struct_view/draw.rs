//! Drawing logic for StructuredView — three-column tree-table.

use txv_core::prelude::*;

use crate::structured::NodeKind;

use super::{ColFocus, StructuredView};

/// Draw the structured view as a three-column tree-table.
pub fn draw_struct_view(view: &mut StructuredView) {
    let w = view.state.buffer_mut().width();
    let h = view.state.buffer_mut().height();
    if w == 0 || h == 0 {
        return;
    }
    let ww = w as usize;
    let key_w = ww * 40 / 100;
    let val_w = ww * 40 / 100;
    let meta_w = ww.saturating_sub(key_w + val_w + 2);

    let normal = Style::default();
    let pal = txv_core::palette::palette();
    let focused = view.state.is_focused();
    let cursor_style = if focused {
        pal.interactive.cursor_focused.to_style()
    } else {
        pal.interactive.cursor_unfocused.to_style()
    };
    let cursor_row_style = normal;
    let sep_style = pal.base.dim.to_style();
    let edit_style = pal.interactive.edit_overlay.to_style();

    // Pre-collect row data to avoid borrow issues
    struct RowData {
        node_id: crate::structured::NodeId,
        is_cursor: bool,
    }
    let rows: Vec<RowData> = (0..h as usize)
        .map(|row| {
            let idx = view.scroll + row;
            RowData {
                node_id: if idx < view.visible_nodes.len() {
                    view.visible_nodes[idx]
                } else {
                    crate::structured::NodeId(0)
                },
                is_cursor: idx == view.cursor,
            }
        })
        .collect();

    #[allow(clippy::needless_range_loop)]
    for row in 0..h as usize {
        let idx = view.scroll + row;
        let y = row as u16;

        if idx >= view.visible_nodes.len() {
            view.state.buffer_mut().hline(0, y, w, ' ', normal);
            continue;
        }

        let node_id = rows[row].node_id;
        let is_cursor = rows[row].is_cursor;
        let base = if is_cursor {
            cursor_row_style
        } else {
            normal
        };

        view.state.buffer_mut().hline(0, y, w, ' ', base);

        // Key column
        let key_text = build_key_text(view, node_id);
        let col_style = if is_cursor && view.col_focus == ColFocus::Key {
            cursor_style
        } else {
            base
        };
        let truncated_key = truncate(&key_text, key_w);
        view.state.buffer_mut().print(0, y, &truncated_key, col_style);

        // Separator 1
        let sep1_x = key_w as u16;
        view.state.buffer_mut().print(sep1_x, y, "│", sep_style);

        // Value column
        let val_text = view.doc.value_display(node_id).to_owned();
        let col_style = if is_cursor && view.col_focus == ColFocus::Value {
            cursor_style
        } else {
            base
        };
        let val_x = sep1_x + 1;
        let truncated_val = truncate(&val_text, val_w);
        view.state.buffer_mut().print(val_x, y, &truncated_val, col_style);

        // Separator 2
        let sep2_x = val_x + val_w as u16;
        view.state.buffer_mut().print(sep2_x, y, "│", sep_style);

        // Meta column
        let meta_text = view.doc.meta(node_id).to_owned();
        let col_style = if is_cursor && view.col_focus == ColFocus::Meta {
            cursor_style
        } else {
            base
        };
        let meta_x = sep2_x + 1;
        if !meta_text.is_empty() && meta_w > 0 {
            let truncated_meta = truncate(&meta_text, meta_w);
            view.state.buffer_mut().print(meta_x, y, &truncated_meta, col_style);
        }

        // Render InlineEditor overlay if editing this row
        if let Some(ref editor) = view.editing {
            if editor.row == idx {
                let (col_x, col_w) = match view.col_focus {
                    ColFocus::Key => (0u16, key_w as u16),
                    ColFocus::Value => (val_x, val_w as u16),
                    ColFocus::Meta => (meta_x, meta_w as u16),
                };
                editor.draw(view.state.buffer_mut(), col_x, y, col_w, edit_style);
            }
        }
    }
}

/// Build the key column text with tree connectors and expand/collapse markers.
fn build_key_text(view: &StructuredView, node_id: crate::structured::NodeId) -> String {
    let depth = view.depth(node_id);
    let mut text = String::new();

    if depth > 0 {
        // Build ancestor continuation lines: │ for non-last ancestors, space for last
        let mut guides = Vec::with_capacity(depth.saturating_sub(1));
        let mut current = node_id;
        for _ in 0..depth.saturating_sub(1) {
            if let Some(parent) = view.doc.parent(current) {
                current = parent;
                guides.push(!view.is_last_child(current));
            }
        }
        guides.reverse();
        for has_line in &guides {
            if *has_line {
                text.push_str("│ ");
            } else {
                text.push_str("  ");
            }
        }
        if view.is_last_child(node_id) {
            text.push_str("└─");
        } else {
            text.push_str("├─");
        }
    }

    let kind = view.doc.node_kind(node_id);
    if kind != NodeKind::Scalar {
        if view.doc.is_expanded(node_id) {
            text.push('▼');
        } else {
            text.push('▶');
        }
        text.push(' ');
    }

    if let Some(key) = view.doc.key(node_id) {
        text.push_str(key);
    } else if depth > 0 {
        if let Some(parent) = view.doc.parent(node_id) {
            let siblings = view.doc.children(parent);
            if let Some(pos) = siblings.iter().position(|&c| c == node_id) {
                text.push_str(&format!("[{pos}]"));
            }
        }
    }

    text
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    }
}
