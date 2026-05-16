//! Key handling for TodoTreeView.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

use super::data::TodoTreeData;
use super::model::{self, Completion, TodoItem};

/// Process todo-specific keys. Returns true if consumed.
pub fn handle_todo_key(key: &KeyEvent, data: &mut TodoTreeData, cursor: usize) -> Option<HandleAction> {
    let id = data.visible_id(cursor);
    match key.code {
        // Shift+Up — swap up (same as K)
        KeyCode::Up if key.modifiers.shift => {
            let path = data.path_at(id)?.clone();
            let new_path = model::swap_up(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // Shift+Down — swap down (same as J)
        KeyCode::Down if key.modifiers.shift => {
            let path = data.path_at(id)?.clone();
            let new_path = model::swap_down(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // Shift+Left — promote (same as H)
        KeyCode::Left if key.modifiers.shift => {
            let path = data.path_at(id)?.clone();
            let new_path = model::promote(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // Shift+Right — demote (same as L)
        KeyCode::Right if key.modifiers.shift => {
            let path = data.path_at(id)?.clone();
            let new_path = model::demote(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // Space — toggle completed
        KeyCode::Char(' ') => {
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
        // ! — toggle important
        KeyCode::Char('!') => {
            let path = data.path_at(id)?.clone();
            let item = model::get_item_mut(&mut data.file, &path)?;
            item.important = !item.important;
            data.save();
            data.rebuild_flat();
            Some(HandleAction::Stay)
        }
        // n — new sibling
        KeyCode::Char('n') => {
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
        // b — new child
        KeyCode::Char('b') => {
            let path = data.path_at(id)?.clone();
            let child_idx = model::get_item(&data.file, &path).map_or(0, |item| item.items.len());
            let new_item = TodoItem::new("<new task>");
            if !model::add_child(&mut data.file, &path, new_item) {
                return Some(HandleAction::Stay);
            }
            // Ensure parent is expanded
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
        // d — delete item (confirm if unchecked or has children)
        KeyCode::Char('d') => {
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
        // S — sort children
        KeyCode::Char('S') => {
            let path = data.path_at(id)?.clone();
            model::sort_children(&mut data.file, &path);
            data.save();
            data.rebuild_flat();
            Some(HandleAction::Stay)
        }
        // c — clone subtree
        KeyCode::Char('c') => {
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
        // / — enter filter mode
        KeyCode::Char('/') => Some(HandleAction::EnterFilter),
        // Ctrl+L — encryption toggle
        KeyCode::Char('l') if key.modifiers.ctrl => {
            let path = data.path_at(id)?.clone();
            let item = model::get_item(&data.file, &path)?;
            if item.is_locked() {
                Some(HandleAction::CryptoDecrypt(path))
            } else if item.is_encrypted() && item.unlocked {
                // Re-lock: no passphrase needed
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
        // J — swap down (Shift+j)
        KeyCode::Char('J') => {
            let path = data.path_at(id)?.clone();
            let new_path = model::swap_down(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // K — swap up (Shift+k)
        KeyCode::Char('K') => {
            let path = data.path_at(id)?.clone();
            let new_path = model::swap_up(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // H — promote (Shift+h)
        KeyCode::Char('H') => {
            let path = data.path_at(id)?.clone();
            let new_path = model::promote(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // L — demote (Shift+l)
        KeyCode::Char('L') => {
            let path = data.path_at(id)?.clone();
            let new_path = model::demote(&mut data.file, &path)?;
            data.save();
            data.rebuild_flat();
            data.row_for_path(&new_path).map(HandleAction::MoveTo)
        }
        // N — open note for this item
        KeyCode::Char('N') => {
            let path = data.path_at(id)?.clone();
            let item = model::get_item(&data.file, &path)?;
            Some(HandleAction::OpenNote(path, item.note.clone()))
        }
        _ => None,
    }
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
}
