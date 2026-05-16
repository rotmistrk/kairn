//! Key handling for CsvView — navigation, sort, filter, edit.

use txv_core::prelude::*;
use txv_widgets::inline_edit::InlineEditResult;

use super::CsvView;
use crate::csv_parse::ColType;

pub fn handle_csv_event(view: &mut CsvView, event: &Event) -> HandleResult {
    let Event::Key(key) = event else {
        return HandleResult::Ignored;
    };

    // Route to inline editor first
    if let Some(ref mut editor) = view.editing {
        match editor.handle_key(key) {
            InlineEditResult::Continue => return HandleResult::Consumed,
            InlineEditResult::Commit(_) => {
                let text = view.editing.take().map(|e| e.buffer).unwrap_or_default();
                commit_edit(view, &text);
                return HandleResult::Consumed;
            }
            InlineEditResult::Cancel => {
                view.editing = None;
                return HandleResult::Consumed;
            }
        }
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if view.cursor_row + 1 < view.visible_rows.len() {
                view.cursor_row += 1;
                ensure_visible(view);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            view.cursor_row = view.cursor_row.saturating_sub(1);
            ensure_visible(view);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if view.cursor_col + 1 < view.ncols() {
                view.cursor_col += 1;
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            view.cursor_col = view.cursor_col.saturating_sub(1);
        }
        KeyCode::Char('g') => {
            view.cursor_row = 0;
            ensure_visible(view);
        }
        KeyCode::Char('G') => {
            view.cursor_row = view.visible_rows.len().saturating_sub(1);
            ensure_visible(view);
        }
        KeyCode::Char('0') => view.cursor_col = 0,
        KeyCode::Char('$') => view.cursor_col = view.ncols().saturating_sub(1),
        KeyCode::Enter => start_edit(view),
        KeyCode::Char('s') => handle_sort(view),
        KeyCode::Char('f') if key.modifiers.ctrl => {
            for f in &mut view.filters {
                f.clear();
            }
            view.refilter();
        }
        KeyCode::Char('f') => handle_filter_start(view),
        KeyCode::Char('F') => {
            if view.cursor_col < view.filters.len() {
                view.filters[view.cursor_col].clear();
                view.refilter();
            }
        }
        KeyCode::Char(':') => {
            view.state.put_command(crate::commands::CM_COMMAND_MODE, None);
        }
        _ => return HandleResult::Ignored,
    }
    HandleResult::Consumed
}

fn start_edit(view: &mut CsvView) {
    if view.visible_rows.is_empty() {
        return;
    }
    let data_idx = view.visible_rows[view.cursor_row];
    let current = view.rows[data_idx].get(view.cursor_col).cloned().unwrap_or_default();
    view.editing = Some(txv_widgets::inline_edit::InlineEditor::new(view.cursor_row, &current));
}

fn commit_edit(view: &mut CsvView, text: &str) {
    if view.visible_rows.is_empty() {
        return;
    }
    let data_idx = view.visible_rows[view.cursor_row];
    // Ensure row has enough columns
    while view.rows[data_idx].len() <= view.cursor_col {
        view.rows[data_idx].push(String::new());
    }
    view.rows[data_idx][view.cursor_col] = text.to_string();
    view.dirty = true;
    // Advance cursor down
    if view.cursor_row + 1 < view.visible_rows.len() {
        view.cursor_row += 1;
        ensure_visible(view);
    }
}

fn handle_sort(view: &mut CsvView) {
    let col = view.cursor_col;
    if view.sort_col == Some(col) {
        view.sort_asc = !view.sort_asc;
    } else {
        view.sort_col = Some(col);
        view.sort_asc = true;
    }
    let asc = view.sort_asc;
    let is_numeric = matches!(view.col_types.get(col), Some(ColType::Numeric { .. }));

    view.visible_rows.sort_by(|&a, &b| {
        let va = view.rows[a].get(col).map(|s| s.as_str()).unwrap_or("");
        let vb = view.rows[b].get(col).map(|s| s.as_str()).unwrap_or("");
        let ord = if is_numeric {
            let na = va.trim().parse::<f64>().ok();
            let nb = vb.trim().parse::<f64>().ok();
            match (na, nb) {
                (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => va.cmp(vb),
            }
        } else {
            va.to_lowercase().cmp(&vb.to_lowercase())
        };
        if asc {
            ord
        } else {
            ord.reverse()
        }
    });
    view.cursor_row = 0;
}

fn handle_filter_start(view: &mut CsvView) {
    view.editing = Some(txv_widgets::inline_edit::InlineEditor::new(
        view.cursor_row,
        &view.filters[view.cursor_col],
    ));
}

fn ensure_visible(view: &mut CsvView) {
    let h = view.state.bounds().h as usize;
    let data_h = h.saturating_sub(if view.headers.is_some() {
        1
    } else {
        0
    });
    if data_h == 0 {
        return;
    }
    if view.cursor_row < view.scroll_row {
        view.scroll_row = view.cursor_row;
    } else if view.cursor_row >= view.scroll_row + data_h {
        view.scroll_row = view.cursor_row - data_h + 1;
    }
}
