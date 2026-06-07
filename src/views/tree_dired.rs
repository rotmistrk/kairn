//! FileTreeView dired (file operations) support.

use std::fs;
use std::path::{Path, PathBuf};

use txv_core::message::Message;
use txv_widgets::{TreeData, CM_STATUS_MESSAGE};

use super::FileTreeView;
use crate::commands::*;
use crate::handler_dired::copy_dir_recursive;

impl FileTreeView {
    /// Return the path at the current cursor position.
    pub(super) fn cursor_path(&mut self) -> Option<PathBuf> {
        if self.inner.cursor() >= self.inner.data_mut().visible_count() {
            return None;
        }
        let cursor = self.inner.cursor();
        let id = self.inner.data_mut().visible_id(cursor);
        Some(self.inner.data_mut().path(id).to_path_buf())
    }

    /// Return the directory context for new file operations.
    fn cursor_dir(&mut self) -> Option<PathBuf> {
        let path = self.cursor_path()?;
        if path.is_dir() {
            Some(path)
        } else {
            path.parent().map(|p| p.to_path_buf())
        }
    }

    /// Map a dired command ID to a prefilled command string, or None.
    pub(super) fn dired_prefill(&mut self, id: u16) -> Option<String> {
        match id {
            CM_TREE_NEW_FILE => {
                let dir = self.cursor_dir().unwrap_or_else(|| self.root.clone());
                Some(format!("new-file {}/", dir.display()))
            }
            CM_TREE_NEW_DIR => {
                let dir = self.cursor_dir().unwrap_or_else(|| self.root.clone());
                Some(format!("new-dir {}/", dir.display()))
            }
            CM_TREE_DELETE => {
                let path = self.cursor_path()?;
                Some(format!("delete-file {}", path.display()))
            }
            CM_TREE_RENAME => {
                let path = self.cursor_path()?;
                Some(format!("rename-file {}", path.display()))
            }
            CM_TREE_COPY => {
                let path = self.cursor_path()?;
                Some(format!("copy-file {}", path.display()))
            }
            _ => None,
        }
    }

    /// Handle mark/unmark/bulk commands directly. Returns true if handled.
    pub(super) fn handle_mark_cmd(&mut self, id: u16) -> bool {
        match id {
            CM_TREE_MARK => {
                if let Some(path) = self.cursor_path() {
                    if self.marked.contains(&path) {
                        self.marked.remove(&path);
                    } else {
                        self.marked.insert(path);
                    }
                    self.inner.mark_dirty();
                }
                true
            }
            CM_TREE_UNMARK_ALL => {
                self.marked.clear();
                self.inner.mark_dirty();
                true
            }
            CM_TREE_MOVE_MARKED | CM_TREE_COPY_MARKED => {
                self.bulk_op(id == CM_TREE_COPY_MARKED);
                true
            }
            _ => false,
        }
    }

    fn bulk_op(&mut self, copy: bool) {
        let dest_dir = match self.cursor_dir() {
            Some(d) => d,
            None => return,
        };
        if self.marked.is_empty() {
            self.inner.state_mut().put_command(
                CM_STATUS_MESSAGE,
                Some(Box::new(Message::warn("file", String::from("No marked files")))),
            );
            return;
        }
        let (ok, errs) = execute_bulk(&mut self.marked, &dest_dir, copy);
        let msg = bulk_message(copy, ok, &errs, &dest_dir);
        self.inner
            .state_mut()
            .put_command(CM_STATUS_MESSAGE, Some(Box::new(Message::info("file", msg))));
        self.inner.state_mut().put_broadcast(CM_FS_CHANGED, None);
    }
}

fn execute_bulk(marked: &mut std::collections::HashSet<PathBuf>, dest_dir: &Path, copy: bool) -> (u16, Vec<String>) {
    let mut ok = 0u16;
    let mut errs = Vec::new();
    for src in marked.drain() {
        let name = match src.file_name() {
            Some(n) => n.to_os_string(),
            None => continue,
        };
        let target = dest_dir.join(&name);
        let result = if copy {
            if src.is_dir() {
                copy_dir_recursive(&src, &target)
            } else {
                fs::copy(&src, &target).map(|_| ())
            }
        } else {
            fs::rename(&src, &target)
        };
        match result {
            Ok(()) => ok += 1,
            Err(e) => errs.push(format!("{}: {e}", src.display())),
        }
    }
    (ok, errs)
}

fn bulk_message(copy: bool, ok: u16, errs: &[String], dest_dir: &Path) -> String {
    let verb = if copy {
        "Copied"
    } else {
        "Moved"
    };
    if errs.is_empty() {
        format!("{verb} {ok} item(s) to {}", dest_dir.display())
    } else {
        format!("{verb} {ok}, failed {}: {}", errs.len(), errs.join("; "))
    }
}
