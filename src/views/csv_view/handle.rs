//! Key handling for CsvView — navigation, sort, filter, edit, row ops.

use std::cmp::Ordering;

use txv_core::prelude::*;

use super::row_ops;
use super::CsvView;
use crate::commands::CM_COMMAND_MODE;
use crate::csv_parse::ColType;

pub fn handle_csv_event(view: &mut CsvView, event: &Event) -> HandleResult {
    let Event::Key(key) = event else {
        return HandleResult::Ignored;
    };
    if key.modifiers().shift() {
        if let Some(r) = handle_shift_motion(view, key.code()) {
            return r;
        }
    }
    match key.code() {
        KeyCode::Down | KeyCode::Char('j') => handle_nav_down(view),
        KeyCode::Up | KeyCode::Char('k') => handle_nav_up(view),
        KeyCode::Right | KeyCode::Char('l') => handle_nav_right(view),
        KeyCode::Left | KeyCode::Char('h') => handle_nav_left(view),
        KeyCode::Char('g') => handle_jump_top(view),
        KeyCode::Char('G') => handle_jump_bottom(view),
        KeyCode::Char('0') => view.cursor_col = 0,
        KeyCode::Char('$') => view.cursor_col = view.ncols().saturating_sub(1),
        KeyCode::Enter => view.start_edit(),
        KeyCode::Char('s') => handle_sort(view),
        KeyCode::Char('f') if key.modifiers().ctrl() => handle_clear_all_filters(view),
        KeyCode::Char('f') => view.start_filter_edit(),
        KeyCode::Char('F') => handle_clear_col_filter(view),
        KeyCode::Char('a') => row_ops::handle_add_row(view),
        KeyCode::Char('d') => row_ops::handle_delete_row(view),
        KeyCode::Char('v') => row_ops::handle_toggle_visual(view),
        KeyCode::Char('y') => row_ops::handle_yank(view),
        KeyCode::Char('p') => row_ops::handle_paste(view),
        KeyCode::Esc => return handle_esc(view),
        KeyCode::Char(':') => view.group.put_command(CM_COMMAND_MODE, None),
        _ => return HandleResult::Ignored,
    }
    HandleResult::Consumed
}

fn handle_shift_motion(view: &mut CsvView, code: KeyCode) -> Option<HandleResult> {
    match code {
        KeyCode::Down | KeyCode::Char('J') => {
            row_ops::start_visual_if_needed(view);
            handle_nav_down(view);
            Some(HandleResult::Consumed)
        }
        KeyCode::Up | KeyCode::Char('K') => {
            row_ops::start_visual_if_needed(view);
            handle_nav_up(view);
            Some(HandleResult::Consumed)
        }
        _ => None,
    }
}

fn handle_esc(view: &mut CsvView) -> HandleResult {
    if view.visual_anchor.is_some() {
        view.visual_anchor = None;
        view.group.mark_dirty();
        HandleResult::Consumed
    } else {
        HandleResult::Ignored
    }
}

pub fn drain_csv_commands(view: &mut CsvView) {
    for ev in view.child_sink.drain() {
        let Event::Command { id, data, .. } = ev else {
            continue;
        };
        match id {
            CM_OK => {
                let text = data
                    .and_then(|d| d.downcast::<String>().ok())
                    .map(|s| *s)
                    .unwrap_or_default();
                if view.editing_filter {
                    commit_filter(view, &text);
                } else {
                    commit_cell(view, &text);
                }
                return;
            }
            CM_CANCEL => {
                view.cancel_edit();
                return;
            }
            _ => {}
        }
    }
}

fn commit_cell(view: &mut CsvView, text: &str) {
    if view.group.child_count() > 0 {
        view.group.remove(0);
    }
    let row = view.editing_row.take().unwrap_or(0);
    view.editing_filter = false;
    let Some(&data_idx) = view.visible_rows.get(row) else {
        return;
    };
    if data_idx >= view.rows.len() {
        return;
    }
    while view.rows[data_idx].len() <= view.cursor_col {
        view.rows[data_idx].push(String::new());
    }
    view.rows[data_idx][view.cursor_col] = text.to_string();
    view.dirty = true;
    if view.cursor_row + 1 < view.visible_rows.len() {
        view.cursor_row += 1;
        ensure_visible(view);
    }
    view.group.mark_dirty();
}

fn commit_filter(view: &mut CsvView, text: &str) {
    if view.group.child_count() > 0 {
        view.group.remove(0);
    }
    view.editing_row = None;
    view.editing_filter = false;
    if view.cursor_col < view.filters.len() {
        view.filters[view.cursor_col] = text.to_string();
    }
    view.refilter();
    view.cursor_row = 0;
    view.group.mark_dirty();
}

fn handle_jump_top(view: &mut CsvView) {
    view.cursor_row = 0;
    ensure_visible(view);
}

fn handle_jump_bottom(view: &mut CsvView) {
    view.cursor_row = view.visible_rows.len().saturating_sub(1);
    ensure_visible(view);
}

fn handle_clear_all_filters(view: &mut CsvView) {
    for f in &mut view.filters {
        f.clear();
    }
    view.refilter();
}

fn handle_clear_col_filter(view: &mut CsvView) {
    if view.cursor_col < view.filters.len() {
        view.filters[view.cursor_col].clear();
        view.refilter();
    }
}

fn handle_nav_down(view: &mut CsvView) {
    if view.cursor_row + 1 < view.visible_rows.len() {
        view.cursor_row += 1;
        ensure_visible(view);
    }
}

fn handle_nav_up(view: &mut CsvView) {
    view.cursor_row = view.cursor_row.saturating_sub(1);
    ensure_visible(view);
}

fn handle_nav_right(view: &mut CsvView) {
    if view.cursor_col + 1 < view.ncols() {
        view.cursor_col += 1;
    }
}

fn handle_nav_left(view: &mut CsvView) {
    view.cursor_col = view.cursor_col.saturating_sub(1);
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
                (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
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
    view.group.mark_dirty();
}

pub(super) fn ensure_visible(view: &mut CsvView) {
    let h = view.group.bounds().h() as usize;
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
