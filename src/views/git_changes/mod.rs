//! GitChangesView — non-closeable tab showing changed files grouped by status.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use crate::commands::*;
use crate::git_watcher::WatchHandle;
use crate::settings::GitKeys;

pub use self::data::GitChangesData;
mod builders;
mod change_node;
mod data;

/// The git changes view — wraps TreeView<GitChangesData>.
pub struct GitChangesView {
    inner: TreeView<GitChangesData>,
    watcher: Option<WatchHandle>,
    root: PathBuf,
    roots: Vec<PathBuf>,
    last_key_was_right: bool,
    keys: GitKeys,
    needs_rebuild: bool,
    tick_counter: u16,
    /// Cooldown ticks after rebuild to avoid feedback loop (status read triggers watcher).
    cooldown: u16,
}

impl GitChangesView {
    pub fn new(root: PathBuf, watcher: Option<WatchHandle>, keys: GitKeys) -> Self {
        let data = GitChangesData::new(&root);
        Self {
            inner: TreeView::new(data),
            watcher,
            roots: vec![root.clone()],
            root,
            last_key_was_right: false,
            keys,
            needs_rebuild: true,
            tick_counter: 0,
            cooldown: 0,
        }
    }

    /// Create with multiple workspace roots.
    pub fn with_roots(roots: Vec<PathBuf>, watcher: Option<WatchHandle>, keys: GitKeys) -> Self {
        let primary = roots.first().cloned().unwrap_or_default();
        let data = GitChangesData::new(&primary);
        Self {
            inner: TreeView::new(data),
            watcher,
            root: primary,
            roots,
            last_key_was_right: false,
            keys,
            needs_rebuild: true,
            tick_counter: 0,
            cooldown: 0,
        }
    }

    /// Get the relative path of the currently selected file (for git operations).
    fn selected_rel_path(&mut self) -> Option<String> {
        let row = self.inner.cursor();
        let id = self.inner.data_mut().visible_id(row);
        let abs = self.inner.data_mut().file_path(id)?;
        // Try each root to find the matching one
        for root in &self.roots {
            if let Ok(rel) = abs.strip_prefix(root) {
                return Some(rel.to_string_lossy().to_string());
            }
        }
        abs.strip_prefix(&self.root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }
}

impl View for GitChangesView {
    delegate_view!(inner, override { title, handle });

    fn title(&self) -> &str {
        "Git"
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            return self.handle_tick();
        }
        if let Event::Command {
            id,
            data,
            broadcast: true,
        } = event
        {
            if *id == CM_ROOTS_CHANGED {
                if let Some(rcd) = data.as_ref().and_then(|d| d.downcast_ref::<RootsChangedData>()) {
                    self.roots = rcd.paths.clone();
                    self.inner.data_mut().set_root_badge_colors(rcd.colors.clone());
                    self.inner.data_mut().set_root_labels(rcd.labels.clone());
                    self.needs_rebuild = true;
                }
                return HandleResult::Ignored;
            }
        }
        if let Event::Command { id, data, .. } = event {
            if *id == CM_OK {
                return self.handle_cm_ok(data);
            }
        }
        if let Event::Key(key) = event {
            if let Some(result) = self.handle_git_key(key) {
                return result;
            }
            self.last_key_was_right = key.code() == KeyCode::Right;
        }
        self.inner.handle(event)
    }
}

impl GitChangesView {
    fn handle_tick(&mut self) -> HandleResult {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        if self.cooldown > 0 {
            self.cooldown -= 1;
            if self.cooldown == 0 {
                if let Some(w) = self.watcher.as_mut() {
                    w.has_changes();
                }
            }
            return HandleResult::Ignored;
        }
        let poll = self.tick_counter.is_multiple_of(60);
        let changed = self.needs_rebuild || poll || self.watcher.as_mut().is_some_and(|w| w.has_changes());
        if changed {
            self.needs_rebuild = false;
            self.inner.data_mut().rebuild_roots(&self.roots);
            self.inner.mark_dirty();
            self.cooldown = 4;
        }
        HandleResult::Ignored
    }

    fn handle_cm_ok(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(&node_id) = boxed.downcast_ref::<usize>() {
                let path = self.inner.data_mut().file_path(node_id).map(|p| p.to_path_buf());
                if let Some(path) = path {
                    let untracked = self.inner.data_mut().is_untracked(node_id);
                    let cmd = if self.last_key_was_right {
                        CM_OPEN_FILE_FOCUS
                    } else {
                        CM_OPEN_FILE
                    };
                    let req = if untracked {
                        OpenFileRequest::new(path)
                    } else {
                        OpenFileRequest::with_diff(path)
                    };
                    self.inner.state_mut().put_command(cmd, Some(Box::new(req)));
                }
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_git_key(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        if *key == self.keys.stage {
            if let Some(rel) = self.selected_rel_path() {
                self.inner.state_mut().put_command(CM_GIT_STAGE, Some(Box::new(rel)));
            }
            return Some(HandleResult::Consumed);
        }
        if *key == self.keys.unstage {
            if let Some(rel) = self.selected_rel_path() {
                self.inner.state_mut().put_command(CM_GIT_UNSTAGE, Some(Box::new(rel)));
            }
            return Some(HandleResult::Consumed);
        }
        if *key == self.keys.untrack {
            if let Some(rel) = self.selected_rel_path() {
                self.inner.state_mut().put_command(CM_GIT_UNTRACK, Some(Box::new(rel)));
            }
            return Some(HandleResult::Consumed);
        }
        if *key == self.keys.commit {
            self.inner.state_mut().put_command(CM_GIT_COMMIT_PROMPT, None);
            return Some(HandleResult::Consumed);
        }
        None
    }
}
