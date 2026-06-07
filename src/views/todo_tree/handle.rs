//! Key handling for TodoTreeView.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

use super::data::TodoTreeData;
use super::model::{self, Completion, TodoItem};

/// Process todo-specific keys. Returns true if consumed.
pub fn handle_todo_key(key: &KeyEvent, data: &mut TodoTreeData, cursor: usize) -> Option<HandleAction> {
    let id = data.visible_id(cursor);
    match key.code() {
        KeyCode::Up if key.modifiers().shift() => handle_shift_move(data, id, model::swap_up),
        KeyCode::Down if key.modifiers().shift() => handle_shift_move(data, id, model::swap_down),
        KeyCode::Left if key.modifiers().shift() => handle_shift_move(data, id, model::promote),
        KeyCode::Right if key.modifiers().shift() => handle_shift_move(data, id, model::demote),
        KeyCode::Char('c') if key.modifiers().ctrl() => handle_copy(data, id),
        KeyCode::Char('v') if key.modifiers().ctrl() => handle_paste(data, id, cursor),
        KeyCode::Char(' ') => handle_toggle_complete(data, id),
        KeyCode::Char('!') => handle_set_priority_5(data, id),
        KeyCode::Char('n') => handle_new_sibling(data, id, cursor),
        KeyCode::Char('b') => handle_new_child(data, id, cursor),
        KeyCode::Char('d') => handle_delete(data, id, cursor),
        KeyCode::Char('S') => handle_sort(data, id),
        KeyCode::Char('c') => handle_clone(data, id, cursor),
        KeyCode::Char('/') => Some(HandleAction::EnterFilter),
        KeyCode::Char('l') if key.modifiers().ctrl() => handle_crypto(data, id),
        KeyCode::Char('J') => handle_shift_move(data, id, model::swap_down),
        KeyCode::Char('K') => handle_shift_move(data, id, model::swap_up),
        KeyCode::Char('H') => handle_shift_move(data, id, model::promote),
        KeyCode::Char('L') => handle_shift_move(data, id, model::demote),
        KeyCode::Enter => handle_open_note(data, id),
        KeyCode::Right if !key.modifiers().shift() => handle_right_expand_or_note(data, id),
        KeyCode::Char('N') => handle_open_note_focus(data, id),
        KeyCode::Char('y') => handle_copy(data, id),
        KeyCode::Char('p') => handle_paste(data, id, cursor),
        _ => None,
    }
}

fn handle_shift_move(
    data: &mut TodoTreeData,
    id: usize,
    op: fn(&mut model::TodoFile, &model::TreePath) -> Option<model::TreePath>,
) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let new_path = op(&mut data.file, &path)?;
    data.save();
    data.rebuild_flat();
    data.row_for_path(&new_path).map(HandleAction::MoveTo)
}

fn handle_toggle_complete(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item_mut(&mut data.file, &path)?;
    item.completed = match item.completed {
        Completion::Done => Completion::Open,
        _ => Completion::Done,
    };
    model::propagate_completion(&mut data.file, &path);
    data.save();
    data.rebuild_flat();
    Some(HandleAction::Stay)
}

fn handle_set_priority_5(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item_mut(&mut data.file, &path)?;
    let current = item.priority.unwrap_or(0);
    item.priority = Some(if current == 5 {
        0
    } else {
        5
    });
    data.save();
    data.rebuild_flat();
    Some(HandleAction::Stay)
}

fn handle_new_sibling(data: &mut TodoTreeData, id: usize, cursor: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let new_item = TodoItem::new("<new task>");
    if !model::add_sibling(&mut data.file, &path, new_item) {
        return Some(HandleAction::Stay);
    }
    let mut new_path = path.clone();
    if let Some(last) = new_path.last_mut() {
        *last += 1;
    }
    model::propagate_completion(&mut data.file, &new_path);
    data.save();
    data.rebuild_flat();
    let row = data.row_for_path(&new_path).unwrap_or(cursor + 1);
    Some(HandleAction::EditNew(row))
}

fn handle_new_child(data: &mut TodoTreeData, id: usize, cursor: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let child_idx = model::get_item(&data.file, &path).map_or(0, |item| item.items.len());
    let new_item = TodoItem::new("<new task>");
    if !model::add_child(&mut data.file, &path, new_item) {
        return Some(HandleAction::Stay);
    }
    if let Some(item) = model::get_item_mut(&mut data.file, &path) {
        item.folded = false;
    }
    let mut new_path = path.clone();
    new_path.push(child_idx);
    model::propagate_completion(&mut data.file, &new_path);
    data.save();
    data.rebuild_flat();
    let row = data.row_for_path(&new_path).unwrap_or(cursor + 1);
    Some(HandleAction::EditNew(row))
}

fn handle_delete(data: &mut TodoTreeData, id: usize, _cursor: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item(&data.file, &path)?;
    let needs_confirm = !item.items.is_empty() || !matches!(item.completed, Completion::Done);
    if !needs_confirm {
        model::remove_item(&mut data.file, &path)?;
        model::propagate_completion(&mut data.file, &path);
        data.save();
        data.rebuild_flat();
        Some(HandleAction::Stay)
    } else {
        Some(HandleAction::ConfirmDelete)
    }
}

fn handle_sort(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    model::sort_children(&mut data.file, &path);
    data.save();
    data.rebuild_flat();
    Some(HandleAction::Stay)
}

fn handle_clone(data: &mut TodoTreeData, id: usize, cursor: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    if model::clone_subtree(&mut data.file, &path) {
        let mut new_path = path.clone();
        if let Some(last) = new_path.last_mut() {
            *last += 1;
        }
        data.save();
        data.rebuild_flat();
        let row = data.row_for_path(&new_path).unwrap_or(cursor + 1);
        Some(HandleAction::MoveTo(row))
    } else {
        Some(HandleAction::Stay)
    }
}

fn handle_crypto(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item(&data.file, &path)?;
    if item.is_locked() {
        Some(HandleAction::CryptoDecrypt(path))
    } else if item.is_encrypted() && item.unlocked {
        if let Some(it) = model::get_item_mut(&mut data.file, &path) {
            it.items.clear();
            it.note.clear();
            it.unlocked = false;
            it.folded = true;
        }
        data.save();
        data.rebuild_flat();
        Some(HandleAction::Stay)
    } else {
        Some(HandleAction::CryptoEncrypt(path))
    }
}

fn handle_open_note(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item(&data.file, &path)?;
    Some(HandleAction::OpenNote(path, item.note.clone()))
}

fn handle_right_expand_or_note(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item_mut(&mut data.file, &path)?;
    if !item.items.is_empty() && item.folded {
        item.folded = false;
        data.save();
        data.rebuild_flat();
        Some(HandleAction::Stay)
    } else {
        let note = item.note.clone();
        Some(HandleAction::OpenNoteFocus(path, note))
    }
}

fn handle_open_note_focus(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?.clone();
    let item = model::get_item(&data.file, &path)?;
    Some(HandleAction::OpenNoteFocus(path, item.note.clone()))
}

/// Action to take after handling a key.
pub enum HandleAction {
    Stay,
    MoveDown,
    MoveTo(usize),
    /// Move to row and open editor with text selected.
    EditNew(usize),
    /// Ask for confirmation before deleting.
    ConfirmDelete,
    /// Enter filter mode.
    EnterFilter,
    /// Encrypt the node at path (prompt for passphrase).
    CryptoEncrypt(model::TreePath),
    /// Decrypt the node at path (prompt for passphrase).
    CryptoDecrypt(model::TreePath),
    /// Open the note editor for the item at path.
    OpenNote(model::TreePath, String),
    /// Open the note editor for the item at path and focus it.
    OpenNoteFocus(model::TreePath, String),
    /// Copy text to system clipboard.
    CopyToClipboard(String),
    /// Paste from clipboard as new sibling.
    PasteFromClipboard,
}

fn handle_copy(data: &mut TodoTreeData, id: usize) -> Option<HandleAction> {
    let path = data.path_at(id)?;
    let item = model::get_item(&data.file, path)?;
    Some(HandleAction::CopyToClipboard(item.title.clone()))
}

fn handle_paste(data: &mut TodoTreeData, id: usize, cursor: usize) -> Option<HandleAction> {
    let _ = (data, id, cursor);
    Some(HandleAction::PasteFromClipboard)
}
