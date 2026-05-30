//! Drawing logic for CsvView — header, grid, cursor, inline editor.

use txv_core::buffer::Buffer;
use txv_core::cell::Style;
use txv_core::geometry::Rect;
use txv_core::palette::{palette, StyleId};

use super::CsvView;

struct DrawStyles {
    normal: Style,
    header: Style,
    cursor: Style,
    cursor_row: Style,
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
        cursor_row: pal.style(StyleId::Dim),
    };

    let mut y: u16 = 0;
    if let Some(ref hdrs) = view.headers {
        let hdrs = hdrs.clone();
        draw_row(
            view.group.buffer_mut(),
            (y, w),
            &view.col_widths,
            &hdrs,
            styles.header,
            (view.scroll_col, usize::MAX),
            &styles,
        );
        y += 1;
    }
    draw_data_rows(view, w, h, y, &styles);
    blit_editor(view, w, h, y);
}

fn draw_data_rows(view: &mut CsvView, w: u16, h: u16, y: u16, styles: &DrawStyles) {
    let data_h = (h as usize).saturating_sub(if view.headers.is_some() {
        1
    } else {
        0
    });
    for row_offset in 0..data_h {
        let vis_idx = view.scroll_row + row_offset;
        let screen_y = y + row_offset as u16;
        if vis_idx >= view.visible_rows.len() {
            view.group.buffer_mut().hline(0, screen_y, w, ' ', styles.normal);
            continue;
        }
        let data_idx = view.visible_rows[vis_idx];
        let is_cursor = vis_idx == view.cursor_row;
        let base = if is_cursor {
            styles.cursor_row
        } else {
            styles.normal
        };
        let cursor_col = if is_cursor {
            view.cursor_col
        } else {
            usize::MAX
        };
        let row_data = view.rows[data_idx].clone();
        draw_row(
            view.group.buffer_mut(),
            (screen_y, w),
            &view.col_widths,
            &row_data,
            base,
            (view.scroll_col, cursor_col),
            styles,
        );
    }
}

fn draw_row(
    buf: &mut Buffer,
    pos: (u16, u16),
    col_widths: &[u16],
    cells: &[String],
    base: Style,
    cols: (usize, usize),
    styles: &DrawStyles,
) {
    let (y, max_w) = pos;
    let (scroll_col, cursor_col) = cols;
    buf.hline(0, y, max_w, ' ', base);
    let sep_style = styles.cursor_row;
    let mut cx: u16 = 0;
    for (col_idx, &width) in col_widths.iter().enumerate() {
        if col_idx < scroll_col {
            continue;
        }
        if cx >= max_w {
            break;
        }
        let remaining = max_w - cx - 1;
        let col_w = width.min(remaining) as usize;
        let style = if col_idx == cursor_col {
            styles.cursor
        } else {
            base
        };
        let cell_text = cells.get(col_idx).map(|s| s.as_str()).unwrap_or("");
        let formatted = format_cell(cell_text, col_w);
        buf.print(cx, y, &formatted, style);
        cx += col_w as u16 + 1;
        if cx <= max_w {
            buf.print(cx - 1, y, "│", sep_style);
        }
    }
}

/// Position and blit the InputLine editor at the cursor cell.
fn blit_editor(view: &mut CsvView, _w: u16, h: u16, header_offset: u16) {
    if view.group.child_count() == 0 {
        return;
    }
    let Some(row) = view.editing_row else {
        return;
    };
    if row < view.scroll_row {
        return;
    }
    let screen_row = (row - view.scroll_row) as u16 + header_offset;
    if screen_row >= h {
        return;
    }
    // Compute x position of cursor_col
    let mut cx: u16 = 0;
    for (col_idx, &width) in view.col_widths.iter().enumerate() {
        if col_idx < view.scroll_col {
            continue;
        }
        if col_idx == view.cursor_col {
            break;
        }
        cx += width + 1;
    }
    let col_w = view.col_widths.get(view.cursor_col).copied().unwrap_or(10);
    let gb = view.group.bounds();
    // Bounds must be absolute for correct cursor translation
    view.group
        .set_child_bounds(0, Rect::new(gb.x + cx, gb.y + screen_row, col_w, 1));
    if let Some(child) = view.group.child_mut(0) {
        child.draw();
    }
    // Blit at buffer-relative position
    let buf_ptr = view.group.buffer_mut() as *mut Buffer;
    if let Some(child) = view.group.child(0) {
        unsafe { (*buf_ptr).blit(child.buffer(), cx, screen_row) };
    }
}

fn format_cell(text: &str, width: usize) -> String {
    let truncated = if text.len() > width {
        &text[..width]
    } else {
        text
    };
    format!("{:<width$}", truncated, width = width)
}
