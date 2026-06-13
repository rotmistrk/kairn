//! Drawing logic for CsvView — header, grid, cursor, inline editor.

use txv_core::buffer::Buffer;
use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};

use super::format::{format_cell, format_numeric_cell, RowContext};
use super::CsvView;
use crate::csv_parse::ColType;

struct DrawStyles {
    normal: Style,
    header: Style,
    cursor: Style,
    cursor_row: Style,
    visual: Style,
}

pub fn draw_csv_view(view: &mut CsvView) {
    let w = view.group.buffer_mut().width();
    let h = view.group.buffer_mut().height();
    if w == 0 || h == 0 {
        return;
    }
    let pal = palette();
    let styles = DrawStyles {
        normal: Style::default(),
        header: pal.style(StyleId::Bright),
        cursor: if view.group.is_focused() {
            pal.style(StyleId::CursorFocused)
        } else {
            pal.style(StyleId::CursorUnfocused)
        },
        cursor_row: if view.group.is_focused() {
            pal.style(StyleId::TableRowActive)
        } else {
            pal.style(StyleId::TableRowInactive)
        },
        visual: pal.style(StyleId::VisualSelection),
    };

    let mut y: u16 = 0;
    if let Some(ref hdrs) = view.headers {
        let hdrs = hdrs.clone();
        let row_ctx = RowContext {
            col_widths: &view.col_widths,
            col_types: &view.col_types,
            cells: &hdrs,
            base: styles.header,
            scroll_col: view.scroll_col,
            cursor_col: usize::MAX,
        };
        draw_row(view.group.buffer_mut(), y, w, &row_ctx, &styles);
        y += 1;
    }
    draw_data_rows(view, w, h, y, &styles);
}

fn draw_data_rows(view: &mut CsvView, w: u16, h: u16, y: u16, styles: &DrawStyles) {
    let data_h = (h as usize).saturating_sub(if view.headers.is_some() {
        1
    } else {
        0
    });
    let visual_range = view.visual_range();
    for row_offset in 0..data_h {
        let vis_idx = view.scroll_row + row_offset;
        let screen_y = y + row_offset as u16;
        if vis_idx >= view.visible_rows.len() {
            view.group.buffer_mut().hline(0, screen_y, w, ' ', styles.normal);
            continue;
        }
        let data_idx = view.visible_rows[vis_idx];
        let is_cursor = vis_idx == view.cursor_row;
        let in_visual = visual_range.is_some_and(|(a, b)| vis_idx >= a && vis_idx <= b);
        let base = if is_cursor {
            styles.cursor_row
        } else if in_visual {
            styles.visual
        } else {
            styles.normal
        };
        let cursor_col = if is_cursor {
            view.cursor_col
        } else {
            usize::MAX
        };
        let row_data = view.rows[data_idx].clone();
        let row_ctx = RowContext {
            col_widths: &view.col_widths,
            col_types: &view.col_types,
            cells: &row_data,
            base,
            scroll_col: view.scroll_col,
            cursor_col,
        };
        draw_row(view.group.buffer_mut(), screen_y, w, &row_ctx, styles);
    }
}

fn draw_row(buf: &mut Buffer, y: u16, max_w: u16, row: &RowContext, styles: &DrawStyles) {
    buf.hline(0, y, max_w, ' ', row.base);
    let mut cx: u16 = 0;
    for (col_idx, &width) in row.col_widths.iter().enumerate() {
        if col_idx < row.scroll_col {
            continue;
        }
        if cx >= max_w {
            break;
        }
        let remaining = max_w - cx - 1;
        let col_w = width.min(remaining) as usize;
        let style = if col_idx == row.cursor_col {
            styles.cursor
        } else {
            row.base
        };
        let cell_text = row.cells.get(col_idx).map(|s| s.as_str()).unwrap_or("");
        let formatted = if matches!(row.col_types.get(col_idx), Some(ColType::Numeric { .. })) {
            format_numeric_cell(cell_text, col_w, &row.col_types[col_idx])
        } else {
            format_cell(cell_text, col_w, false)
        };
        buf.print(cx, y, &formatted, style);
        cx += col_w as u16 + 1;
        if cx <= max_w {
            buf.print(cx - 1, y, "│", styles.cursor_row);
        }
    }
}
