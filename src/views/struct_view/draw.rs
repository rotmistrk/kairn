//! Drawing logic for StructuredView — three-column tree-table.

use txv_core::palette::{palette, StyleId};
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
    let pal = palette();
    let focused = view.state.is_focused();
    let cursor_style = if focused {
        pal.style(StyleId::CursorFocused)
    } else {
        pal.style(StyleId::CursorUnfocused)
    };
    let sep_style = pal.style(StyleId::Dim);
    let edit_style = pal.style(StyleId::EditOverlay);

    let cols = ColLayout { key_w, val_w, meta_w };

    #[allow(clippy::needless_range_loop)]
    for row in 0..h as usize {
        let idx = view.scroll + row;
        let y = row as u16;
        if idx >= view.visible_nodes.len() {
            view.state.buffer_mut().hline(0, y, w, ' ', normal);
            continue;
        }
        let node_id = view.visible_nodes[idx];
        let is_cursor = idx == view.cursor;
        draw_struct_row(view, y, node_id, is_cursor, (normal, cursor_style, sep_style), &cols);
    }
    draw_edit_overlay(view, h, edit_style, &cols);
}

struct ColLayout {
    key_w: usize,
    val_w: usize,
    meta_w: usize,
}

fn draw_struct_row(
    view: &mut StructuredView,
    y: u16,
    node_id: crate::structured::NodeId,
    is_cursor: bool,
    row_styles: (Style, Style, Style),
    cols: &ColLayout,
) {
    let (normal, cursor_style, sep_style) = row_styles;
    let w = view.state.buffer_mut().width();
    view.state.buffer_mut().hline(0, y, w, ' ', normal);

    let key_text = build_key_text(view, node_id);
    let st = pick_col_style(is_cursor, ColFocus::Key, view.col_focus, cursor_style, normal);
    view.state
        .buffer_mut()
        .print(0, y, &truncate(&key_text, cols.key_w), st);

    let sep1_x = cols.key_w as u16;
    view.state.buffer_mut().print(sep1_x, y, "│", sep_style);

    let val_text = view.doc.value_display(node_id).to_owned();
    let st = pick_col_style(is_cursor, ColFocus::Value, view.col_focus, cursor_style, normal);
    let val_x = sep1_x + 1;
    view.state
        .buffer_mut()
        .print(val_x, y, &truncate(&val_text, cols.val_w), st);

    let sep2_x = val_x + cols.val_w as u16;
    view.state.buffer_mut().print(sep2_x, y, "│", sep_style);

    let meta_text = view.doc.meta(node_id).to_owned();
    let st = pick_col_style(is_cursor, ColFocus::Meta, view.col_focus, cursor_style, normal);
    let meta_x = sep2_x + 1;
    if !meta_text.is_empty() && cols.meta_w > 0 {
        view.state
            .buffer_mut()
            .print(meta_x, y, &truncate(&meta_text, cols.meta_w), st);
    }
}

fn pick_col_style(is_cursor: bool, target: ColFocus, current: ColFocus, cursor: Style, base: Style) -> Style {
    if is_cursor && current == target {
        cursor
    } else {
        base
    }
}

fn draw_edit_overlay(view: &mut StructuredView, h: u16, edit_style: Style, cols: &ColLayout) {
    if let Some(ref editor) = view.editing {
        let idx = editor.row;
        if idx >= view.scroll && idx < view.scroll + h as usize {
            let y = (idx - view.scroll) as u16;
            let sep1_x = cols.key_w as u16;
            let val_x = sep1_x + 1;
            let sep2_x = val_x + cols.val_w as u16;
            let meta_x = sep2_x + 1;
            let (col_x, col_w) = match view.col_focus {
                ColFocus::Key => (0u16, cols.key_w as u16),
                ColFocus::Value => (val_x, cols.val_w as u16),
                ColFocus::Meta => (meta_x, cols.meta_w as u16),
            };
            editor.draw(view.state.buffer_mut(), col_x, y, col_w, edit_style);
        }
    }
}

/// Build the key column text with tree connectors and expand/collapse markers.
fn build_key_text(view: &StructuredView, node_id: crate::structured::NodeId) -> String {
    let depth = view.depth(node_id);
    let mut text = build_tree_guides(view, node_id, depth);
    append_expand_marker(view, node_id, &mut text);
    append_key_label(view, node_id, depth, &mut text);
    text
}

fn build_tree_guides(view: &StructuredView, node_id: crate::structured::NodeId, depth: usize) -> String {
    let mut text = String::new();
    if depth > 0 {
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
            text.push_str(if *has_line {
                "│ "
            } else {
                "  "
            });
        }
        text.push_str(if view.is_last_child(node_id) {
            "└─"
        } else {
            "├─"
        });
    }
    text
}

fn append_expand_marker(view: &StructuredView, node_id: crate::structured::NodeId, text: &mut String) {
    let kind = view.doc.node_kind(node_id);
    if kind != NodeKind::Scalar {
        text.push(if view.doc.is_expanded(node_id) {
            '▼'
        } else {
            '▶'
        });
        text.push(' ');
    }
}

fn append_key_label(view: &StructuredView, node_id: crate::structured::NodeId, depth: usize, text: &mut String) {
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
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    }
}
