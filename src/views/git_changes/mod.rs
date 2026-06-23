//! GitChangesView — non-closeable tab showing changed files grouped by status.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use txv_core::prelude::*;
use txv_core::run::Waker;
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use crate::commands::*;
use crate::git_status_async::{git_status_async, GitStatusTask};
use crate::git_status_params::GitStatusParams;
use crate::git_watcher::WatchHandle;
use crate::settings::GitKeys;

pub use self::data::GitChangesData;
pub(crate) mod builders;
pub(crate) mod change_node;
mod data;
mod handle;

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
    /// Git diff base commits per root.
    diff_base: HashMap<PathBuf, String>,
    /// Dynamic title.
    display_title: String,
    /// In-flight async git status task.
    task: Option<Arc<GitStatusTask>>,
    /// Waker for async notification.
    waker: Waker,
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
            diff_base: HashMap::new(),
            display_title: "Git".to_string(),
            task: None,
            waker: Waker::noop(),
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
            diff_base: HashMap::new(),
            display_title: "Git".to_string(),
            task: None,
            waker: Waker::noop(),
        }
    }

    /// Set the waker for async notifications.
    pub fn set_waker(&mut self, waker: Waker) {
        self.waker = waker;
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
    delegate_view!(inner, override { title, handle, cursor });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn cursor(&self) -> Option<CursorRequest> {
        None
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
            if *id == CM_GIT_BASE_CHANGED {
                self.apply_diff_base(data);
                return HandleResult::Consumed;
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
    fn apply_diff_base(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let new_base = data
            .as_ref()
            .and_then(|d| d.downcast_ref::<HashMap<PathBuf, String>>())
            .cloned()
            .unwrap_or_default();
        self.diff_base = new_base;
        self.display_title = format_git_title(&self.diff_base);
        self.inner.data_mut().set_diff_base(self.diff_base.clone());
        self.needs_rebuild = true;
    }

    fn handle_tick(&mut self) -> HandleResult {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        // Check if async task completed
        if let Some(task) = &self.task {
            if task.is_done() {
                let nodes = task.take_nodes();
                self.task = None;
                self.inner.data_mut().apply_nodes(nodes);
                self.inner.mark_dirty();
                self.cooldown = 4;
                return HandleResult::Ignored;
            }
        }
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
        if changed && self.task.is_none() {
            self.needs_rebuild = false;
            self.spawn_rebuild();
        }
        HandleResult::Ignored
    }

    fn spawn_rebuild(&mut self) {
        if let Some(old) = self.task.take() {
            old.cancel();
        }
        let collapsed = self.inner.data_mut().collapsed_keys();
        let params = GitStatusParams {
            roots: self.roots.clone(),
            diff_base: self.diff_base.clone(),
            root_badge_colors: self.inner.data_mut().badge_colors().to_vec(),
            root_labels: self.inner.data_mut().labels().to_vec(),
            collapsed,
        };
        self.task = Some(git_status_async(params, self.waker.clone()));
    }
}

fn format_git_title(_base: &HashMap<PathBuf, String>) -> String {
    "Git".to_string()
}
