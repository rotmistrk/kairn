//! Drawing logic for CsvView — header, grid, cursor, filters.

use txv_core::cell::Style;
use txv_core::prelude::*;

use super::CsvView;
use crate::csv_parse::ColType;

/// Styles needed for rendering a CSV grid.
struct DrawStyles {
    normal: Style,
    header: Style,
    cursor: Style,
    cursor_row: Style,
}

pub fn draw_csv_view(view: &CsvView, surface: &mut Surface) {
    let b = view.state.bounds();
    if b.w == 0 || b.h == 0 {
        return;
    }
    let pal = txv_core::palette::palette();
    let styles = DrawStyles {
        normal: Style::default(),
        header: pal.base.bright.to_style(),
        cursor: if view.state.is_focused() {
            pal.interactive.cursor_focused.to_style()
        } else {
            pal.interactive.cursor_unfocused.to_style()
        },
        cursor_row: pal.base.dim.to_style(),
    };

    let mut y = b.y;
    let avail_h = b.h as usize;

    // Header row (frozen)
    if let Some(ref hdrs) = view.headers {
        draw_row(surface, b.x, y, b.w, hdrs, view, styles.header, usize::MAX, &styles);
        y += 1;
    }

    // Data rows
    let data_h = avail_h.saturating_sub(if view.headers.is_some() {
        1
    } else {
        0
    });
    for row_offset in 0..data_h {
        let vis_idx = view.scroll_row + row_offset;
        let screen_y = y + row_offset as u16;
        if vis_idx >= view.visible_rows.len() {
            surface.hline(b.x, screen_y, b.w, ' ', styles.normal);
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
        draw_row(
            surface,
            b.x,
            screen_y,
            b.w,
            &view.rows[data_idx],
            view,
            base,
            cursor_col,
            &styles,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_row(
    surface: &mut Surface,
    x: u16,
    y: u16,
    max_w: u16,
    cells: &[String],
    view: &CsvView,
    base: Style,
    cursor_col: usize,
    styles: &DrawStyles,
) {
    surface.hline(x, y, max_w, ' ', base);
    let sep_style = styles.cursor_row;
    let mut cx = x;
    for (col_idx, &width) in view.col_widths.iter().enumerate() {
        if col_idx < view.scroll_col {
            continue;
        }
        if cx >= x + max_w {
            break;
        }
        let remaining = x + max_w - cx - 1;
        let col_w = width.min(remaining) as usize;
        let style = if col_idx == cursor_col {
            styles.cursor
        } else {
            base
        };
        let cell_text = cells.get(col_idx).map(|s| s.as_str()).unwrap_or("");
        let formatted = format_cell(cell_text, col_w, &view.col_types, col_idx);
        surface.print(cx, y, &formatted, style);
        cx += col_w as u16 + 1;
        if cx <= x + max_w {
            surface.print(cx - 1, y, "│", sep_style);
        }
    }
}

fn format_cell(text: &str, width: usize, col_types: &[ColType], col_idx: usize) -> String {
    let truncated = if text.len() > width {
        &text[..width]
    } else {
        text
    };
    match col_types.get(col_idx) {
        Some(ColType::Numeric { .. }) => format!("{:>width$}", truncated, width = width),
        _ => format!("{:<width$}", truncated, width = width),
    }
}
