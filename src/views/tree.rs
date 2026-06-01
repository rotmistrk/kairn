//! FileTreeView — wraps TreeView<FileTreeData>, emits CM_OPEN_FILE on Enter.

#[path = "tree_dired.rs"]
mod tree_dired;

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::cell::Color;
use txv_core::cursor::{CursorRequest, CursorShape};
use txv_core::prelude::*;
use txv_widgets::{FileTreeData, TreeView};

use crate::app_palette::app_palette;
use crate::commands::{RootsChangedData, CM_COMMAND_PREFILL, CM_FS_CHANGED, CM_OPEN_FILES_CHANGED, CM_ROOTS_CHANGED};
use std::collections::HashSet;

use crate::git_status::{collect_git_status, FileStatus};
use crate::git_watcher::WatchHandle;

pub struct FileTreeView {
    pub(super) inner: TreeView<FileTreeData>,
    pub(super) last_key_was_right: bool,
    pub(super) watcher: Option<WatchHandle>,
    pub(super) root: PathBuf,
    pub(super) refresh_counter: u16,
    pub(super) filter_active: bool,
    pub(super) marked: HashSet<PathBuf>,
}

impl FileTreeView {
    pub fn new(root: PathBuf, watcher: Option<WatchHandle>) -> Self {
        let data = FileTreeData::new(root.clone());
        let mut view = Self {
            inner: TreeView::new(data),
            last_key_was_right: false,
            watcher,
            root,
            refresh_counter: 0,
            filter_active: false,
            marked: HashSet::new(),
        };
        view.update_colors();
        view
    }

    /// Create a multi-root file tree view.
    pub fn with_roots(roots: Vec<PathBuf>, watcher: Option<WatchHandle>) -> Self {
        let primary = roots.first().cloned().unwrap_or_default();
        let data = FileTreeData::with_roots(roots);
        let mut view = Self {
            inner: TreeView::new(data),
            last_key_was_right: false,
            watcher,
            root: primary,
            refresh_counter: 0,
            filter_active: false,
            marked: HashSet::new(),
        };
        view.update_colors();
        view
    }

    pub(super) fn update_colors(&mut self) {
        let mut colors: HashMap<String, Color> = HashMap::new();
        for root in self.inner.data.all_roots() {
            for (path, status) in collect_git_status(root) {
                colors.insert(path, status_color(status));
            }
        }
        self.inner.data.set_colors(colors);
    }

    /// Signal that a save occurred (immediate refresh trigger).
    pub fn notify_save(&self) {
        if let Some(w) = &self.watcher {
            w.signal_change();
        }
    }

    pub(super) fn clear_filter(&mut self) {
        if self.filter_active {
            self.filter_active = false;
            self.inner.data.set_filter("");
            self.inner.mark_dirty();
        }
    }

    /// Return paths of all expanded directories.
    pub fn expanded_paths(&self) -> Vec<PathBuf> {
        self.inner.data.expanded_paths()
    }

    /// Expand directories matching the given paths.
    pub fn expand_paths(&mut self, paths: &[PathBuf]) {
        self.inner.data.expand_paths(paths);
    }
}

fn status_color(status: FileStatus) -> Color {
    let app = app_palette();
    match status {
        FileStatus::Modified => app.git().modified().fg,
        FileStatus::Added => app.git().added().fg,
        FileStatus::Untracked => app.git().untracked().fg,
        FileStatus::Ignored => app.git().ignored().fg,
        FileStatus::Conflict => app.git().conflict().fg,
        FileStatus::Clean => Color::Reset,
    }
}

impl View for FileTreeView {
    delegate_view!(inner, override { title, handle, unselect, can_close });

    fn title(&self) -> &str {
        "Files"
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if !self.filter_active {
            return None;
        }
        Some(CursorRequest {
            x: 1 + self.inner.data.filter().len() as u16,
            y: self.inner.bounds().h.saturating_sub(1),
            shape: CursorShape::Bar,
        })
    }

    fn unselect(&mut self) {
        self.clear_filter();
        self.inner.unselect();
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
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
            return self.handle_broadcast(*id, data);
        }
        if let Event::Key(key) = event {
            self.last_key_was_right = key.code == KeyCode::Right;
            if let Some(result) = self.handle_filter_key(key) {
                return result;
            }
        }
        if let Event::Command { id, data, .. } = event {
            if self.handle_mark_cmd(*id) {
                return HandleResult::Consumed;
            }
            if let Some(prefill) = self.dired_prefill(*id) {
                self.inner
                    .state
                    .put_command(CM_COMMAND_PREFILL, Some(Box::new(prefill)));
                return HandleResult::Consumed;
            }
            if *id == CM_OK {
                return self.handle_cm_ok(data);
            }
        }
        self.inner.handle(event)
    }
}

impl FileTreeView {
    fn handle_broadcast(&mut self, id: u16, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if id == CM_FS_CHANGED {
            return self.handle_save();
        }
        if id == CM_ROOTS_CHANGED {
            if let Some(rcd) = data.as_ref().and_then(|d| d.downcast_ref::<RootsChangedData>()) {
                self.inner.data.set_roots(rcd.paths.clone());
                self.inner.data.set_root_labels(&rcd.labels);
                self.inner.data.set_root_badge_colors(rcd.colors.clone());
                self.inner.mark_dirty();
                self.update_colors();
            }
            return HandleResult::Ignored;
        }
        if id == CM_OPEN_FILES_CHANGED {
            if let Some(set) = data.as_ref().and_then(|d| d.downcast_ref::<HashSet<PathBuf>>()) {
                self.inner.data.set_open_files(set.clone());
                self.inner.mark_dirty();
            }
            return HandleResult::Ignored;
        }
        HandleResult::Ignored
    }
}
