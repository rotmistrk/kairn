//! Key handling for TodoTreeView.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

use super::data::TodoTreeData;
use super::model::{self, Completion, TodoItem};

/// Process todo-specific keys. Returns true if consumed.
pub fn handle_todo_key(
    key: &KeyEvent,
    data: &mut TodoTreeData,
    cursor: usize,
    queue: &mut EventQueue,
) -> Option<HandleAction> {
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
            let new_item = TodoItem::new("New task");
            model::add_sibling(&mut data.file, &path, new_item);
            model::propagate_completion(&mut data.file, &path);
            data.save();
            data.rebuild_flat();
            // New sibling is at cursor+1
            Some(HandleAction::EditNew(cursor + 1))
        }
        // b — new child
        KeyCode::Char('b') => {
            let path = data.path_at(id)?.clone();
            let new_item = TodoItem::new("New subtask");
            model::add_child(&mut data.file, &path, new_item);
            // Ensure parent is expanded
            let item = model::get_item_mut(&mut data.file, &path)?;
            item.folded = false;
            model::propagate_completion(&mut data.file, &path);
            data.save();
            data.rebuild_flat();
            // New child is right after parent
            Some(HandleAction::EditNew(cursor + 1))
        }
        // d — delete item (confirm if unchecked)
        KeyCode::Char('d') => {
            let path = data.path_at(id)?.clone();
            let is_done =
                model::get_item(&data.file, &path).is_some_and(|item| matches!(item.completed, Completion::Done));
            if is_done {
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
                data.save();
                data.rebuild_flat();
                Some(HandleAction::MoveTo(cursor + 1))
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
            let _ = queue;
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
}
