//! Row operations for CsvView — add, delete, visual select, yank, paste.

use super::CsvView;
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};

use super::handle::ensure_visible;

pub(super) fn start_visual_if_needed(view: &mut CsvView) {
    if view.visual_anchor.is_none() {
        view.visual_anchor = Some(view.cursor_row);
    }
}

pub(super) fn handle_toggle_visual(view: &mut CsvView) {
    if view.visual_anchor.is_some() {
        view.visual_anchor = None;
    } else {
        view.visual_anchor = Some(view.cursor_row);
    }
    view.group.mark_dirty();
}

pub(super) fn handle_add_row(view: &mut CsvView) {
    let ncols = view.ncols();
    let new_row = vec![String::new(); ncols];
    let insert_pos = if view.visible_rows.is_empty() {
        view.rows.len()
    } else {
        let vis = view.cursor_row.min(view.visible_rows.len().saturating_sub(1));
        view.visible_rows[vis] + 1
    };
    view.rows.insert(insert_pos, new_row);
    view.dirty = true;
    view.refilter();
    if let Some(new_vis) = view.visible_rows.iter().position(|&i| i == insert_pos) {
        view.cursor_row = new_vis;
    }
    ensure_visible(view);
    view.group.mark_dirty();
}

pub(super) fn handle_delete_row(view: &mut CsvView) {
    if view.visible_rows.is_empty() {
        return;
    }
    let (start, end) = view.visual_range().unwrap_or((view.cursor_row, view.cursor_row));
    let count = end - start + 1;
    let msg = if count == 1 {
        "Delete row? [y]es [Esc]cancel".to_string()
    } else {
        format!("Delete {count} rows? [y]es [Esc]cancel")
    };
    view.group
        .put_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ConfirmContext::CsvDeleteRow)));
    view.group.put_command(CM_CONFIRM, Some(Box::new(msg)));
}

/// Actually delete rows after confirmation.
pub fn execute_delete(view: &mut CsvView) {
    let (start, end) = view.visual_range().unwrap_or((view.cursor_row, view.cursor_row));
    let mut data_indices: Vec<usize> = (start..=end)
        .filter_map(|vis| view.visible_rows.get(vis).copied())
        .collect();
    data_indices.sort_unstable();
    data_indices.dedup();
    for &idx in data_indices.iter().rev() {
        if idx < view.rows.len() {
            view.rows.remove(idx);
        }
    }
    view.dirty = true;
    view.visual_anchor = None;
    view.refilter();
    view.cursor_row = view.cursor_row.min(view.visible_rows.len().saturating_sub(1));
    ensure_visible(view);
    view.group.mark_dirty();
}

pub(super) fn handle_yank(view: &mut CsvView) {
    if view.visible_rows.is_empty() {
        return;
    }
    let (start, end) = view.visual_range().unwrap_or((view.cursor_row, view.cursor_row));
    let rows: Vec<Vec<String>> = (start..=end)
        .filter_map(|vis| view.visible_rows.get(vis).copied())
        .map(|idx| view.rows[idx].clone())
        .collect();
    if let Some(ref clip) = view.clipboard {
        let text = rows
            .iter()
            .map(|r| r.join(&view.delimiter.to_string()))
            .collect::<Vec<_>>()
            .join("\n");
        if let Ok(mut ring) = clip.lock() {
            ring.push(&text, "csv");
        }
    }
    view.yanked_rows = rows;
    view.visual_anchor = None;
    view.group.mark_dirty();
}

pub(super) fn handle_paste(view: &mut CsvView) {
    if view.yanked_rows.is_empty() {
        return;
    }
    let insert_pos = if view.visible_rows.is_empty() {
        view.rows.len()
    } else {
        let vis = view.cursor_row.min(view.visible_rows.len().saturating_sub(1));
        view.visible_rows[vis] + 1
    };
    for (i, row) in view.yanked_rows.iter().enumerate() {
        view.rows.insert(insert_pos + i, row.clone());
    }
    view.dirty = true;
    view.refilter();
    if let Some(new_vis) = view.visible_rows.iter().position(|&i| i == insert_pos) {
        view.cursor_row = new_vis;
    }
    ensure_visible(view);
    view.group.mark_dirty();
}
