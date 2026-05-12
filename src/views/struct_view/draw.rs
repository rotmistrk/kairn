//! Drawing logic for StructuredView — three-column tree-table.

use txv_core::prelude::*;

use crate::structured::NodeKind;

use super::{ColFocus, StructuredView};

/// Draw the structured view as a three-column tree-table.
pub fn draw_struct_view(view: &StructuredView, surface: &mut Surface) {
    let b = view.state.bounds();
    if b.w == 0 || b.h == 0 {
        return;
    }
    let w = b.w as usize;
    let key_w = w * 40 / 100;
    let val_w = w * 40 / 100;
    let meta_w = w.saturating_sub(key_w + val_w + 2); // 2 for separators

    let normal = Style::default();
    let cursor_style = Style {
        bg: Color::Ansi(4),
        ..Style::default()
    };
    let sep_style = Style {
        fg: Color::Ansi(8),
        ..Style::default()
    };
    let focus_style = Style {
        bg: Color::Ansi(4),
        attrs: Attrs {
            underline: true,
            ..Attrs::default()
        },
        ..Style::default()
    };

    for row in 0..b.h as usize {
        let idx = view.scroll + row;
        let y = b.y + row as u16;

        if idx >= view.visible_nodes.len() {
            surface.hline(b.x, y, b.w, ' ', normal);
            continue;
        }

        let node_id = view.visible_nodes[idx];
        let is_cursor = idx == view.cursor;
        let base = if is_cursor {
            cursor_style
        } else {
            normal
        };

        surface.hline(b.x, y, b.w, ' ', base);

        // Key column
        let key_text = build_key_text(view, node_id);
        let col_style = if is_cursor && view.col_focus == ColFocus::Key {
            focus_style
        } else {
            base
        };
        let truncated_key = truncate(&key_text, key_w);
        surface.print(b.x, y, &truncated_key, col_style);

        // Separator 1
        let sep1_x = b.x + key_w as u16;
        surface.print(sep1_x, y, "│", sep_style);

        // Value column
        let val_text = view.doc.value_display(node_id);
        let col_style = if is_cursor && view.col_focus == ColFocus::Value {
            focus_style
        } else {
            base
        };
        let val_x = sep1_x + 1;
        let truncated_val = truncate(val_text, val_w);
        surface.print(val_x, y, &truncated_val, col_style);

        // Separator 2
        let sep2_x = val_x + val_w as u16;
        surface.print(sep2_x, y, "│", sep_style);

        // Meta column
        let meta_text = view.doc.meta(node_id);
        let col_style = if is_cursor && view.col_focus == ColFocus::Meta {
            focus_style
        } else {
            base
        };
        let meta_x = sep2_x + 1;
        if !meta_text.is_empty() && meta_w > 0 {
            let truncated_meta = truncate(meta_text, meta_w);
            surface.print(meta_x, y, &truncated_meta, col_style);
        }
    }
}

/// Build the key column text with tree connectors and expand/collapse markers.
fn build_key_text(view: &StructuredView, node_id: crate::structured::NodeId) -> String {
    let depth = view.depth(node_id);
    let mut text = String::new();

    // Indent with tree connectors
    if depth > 0 {
        for _ in 0..depth.saturating_sub(1) {
            text.push_str("  ");
        }
        if view.is_last_child(node_id) {
            text.push_str("└─");
        } else {
            text.push_str("├─");
        }
    }

    // Expand/collapse marker for containers
    let kind = view.doc.node_kind(node_id);
    if kind != NodeKind::Scalar {
        if view.doc.is_expanded(node_id) {
            text.push('▼');
        } else {
            text.push('▶');
        }
        text.push(' ');
    }

    // Key name or array index
    if let Some(key) = view.doc.key(node_id) {
        text.push_str(key);
    } else if depth > 0 {
        // Array element — show index
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
