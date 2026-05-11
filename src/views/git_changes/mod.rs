//! GitChangesView — non-closeable tab showing changed files grouped by status.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use crate::commands::*;
use crate::git_watcher::WatchHandle;
use crate::settings::GitKeys;

pub use self::data::GitChangesData;
mod data;

/// The git changes view — wraps TreeView<GitChangesData>.
pub struct GitChangesView {
    inner: TreeView<GitChangesData>,
    watcher: Option<WatchHandle>,
    root: PathBuf,
    last_key_was_right: bool,
    keys: GitKeys,
}

impl GitChangesView {
    pub fn new(root: PathBuf, watcher: Option<WatchHandle>, keys: GitKeys) -> Self {
        let data = GitChangesData::new(&root);
        Self {
            inner: TreeView::new(data),
            watcher,
            root,
            last_key_was_right: false,
            keys,
        }
    }

    /// Get the relative path of the currently selected file (for git operations).
    fn selected_rel_path(&self) -> Option<String> {
        let row = self.inner.cursor;
        let id = self.inner.data.visible_id(row);
        let abs = self.inner.data.file_path(id)?;
        abs.strip_prefix(&self.root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }
}

impl View for GitChangesView {
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }
    fn set_bounds(&mut self, r: Rect) {
        self.inner.set_bounds(r);
    }
    fn options(&self) -> ViewOptions {
        self.inner.options()
    }
    fn title(&self) -> &str {
        "Git"
    }
    fn needs_redraw(&self) -> bool {
        self.inner.needs_redraw()
    }
    fn mark_redrawn(&mut self) {
        self.inner.mark_redrawn();
    }
    fn select(&mut self) {
        self.inner.select();
    }
    fn unselect(&mut self) {
        self.inner.unselect();
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn draw(&self, surface: &mut Surface) {
        self.inner.draw(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Tick = event {
            if self.watcher.as_mut().is_some_and(|w| w.has_changes()) {
                self.inner.data.rebuild(&self.root);
                self.inner.state.dirty = true;
            }
            return HandleResult::Ignored;
        }
        // Handle git-specific keys before passing to TreeView
        if let Event::Key(key) = event {
            if *key == self.keys.stage {
                if let Some(rel) = self.selected_rel_path() {
                    queue.put_command(CM_GIT_STAGE, Some(Box::new(rel)));
                }
                return HandleResult::Consumed;
            }
            if *key == self.keys.unstage {
                if let Some(rel) = self.selected_rel_path() {
                    queue.put_command(CM_GIT_UNSTAGE, Some(Box::new(rel)));
                }
                return HandleResult::Consumed;
            }
            if *key == self.keys.untrack {
                if let Some(rel) = self.selected_rel_path() {
                    queue.put_command(CM_GIT_UNTRACK, Some(Box::new(rel)));
                }
                return HandleResult::Consumed;
            }
            if *key == self.keys.commit {
                queue.put_command(CM_GIT_COMMIT_PROMPT, None);
                return HandleResult::Consumed;
            }
            self.last_key_was_right = key.code == KeyCode::Right;
        }
        let result = self.inner.handle(event, queue);
        // Intercept CM_OK from TreeView (Enter/Right on a node)
        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, data } = &ev {
                if *id == CM_OK {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(&node_id) = boxed.downcast_ref::<usize>() {
                            if let Some(path) = self.inner.data.file_path(node_id) {
                                let cmd = if self.last_key_was_right {
                                    CM_OPEN_FILE_FOCUS
                                } else {
                                    CM_OPEN_FILE
                                };
                                let req = OpenFileRequest::new(path.to_path_buf());
                                queue.put_command(cmd, Some(Box::new(req)));
                                continue;
                            }
                        }
                    }
                }
            }
            queue.put(ev);
        }
        result
    }
}
