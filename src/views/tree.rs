//! FileTreeView — wraps TreeView<FileTreeData>, emits CM_OPEN_FILE on Enter.

#[path = "tree_dired.rs"]
mod tree_dired;

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::cell::Color;
use txv_core::prelude::*;
use txv_widgets::{FileTreeData, TreeView};

use crate::commands::{OpenFileRequest, CM_COMMAND_PREFILL, CM_OPEN_FILE, CM_OPEN_FILE_FOCUS, CM_SAVE};
use std::collections::HashSet;

use crate::git_status::{collect_git_status, FileStatus};
use crate::git_watcher::WatchHandle;

pub struct FileTreeView {
    pub(super) inner: TreeView<FileTreeData>,
    last_key_was_right: bool,
    watcher: Option<WatchHandle>,
    pub(super) root: PathBuf,
    refresh_counter: u16,
    filter_active: bool,
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

    fn update_colors(&mut self) {
        let statuses = collect_git_status(&self.root);
        let colors: HashMap<String, Color> = statuses
            .into_iter()
            .map(|(path, status)| (path, status_color(status)))
            .collect();
        self.inner.data.set_colors(colors);
    }

    /// Signal that a save occurred (immediate refresh trigger).
    pub fn notify_save(&self) {
        if let Some(w) = &self.watcher {
            w.signal_change();
        }
    }

    fn clear_filter(&mut self) {
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
    let app = crate::app_palette::app_palette();
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
    delegate_view!(inner, override { title, handle, unselect });

    fn title(&self) -> &str {
        "Files"
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
        if let Event::Command { id: CM_SAVE, .. } = event {
            self.notify_save();
            self.inner.data.refresh();
            self.inner.mark_dirty();
            self.update_colors();
            return HandleResult::Ignored;
        }
        if let Event::Key(key) = event {
            self.last_key_was_right = key.code == KeyCode::Right;
            if let Some(result) = self.handle_filter_key(key) {
                return result;
            }
        }
        if let Event::Command { id, .. } = event {
            if self.handle_mark_cmd(*id) {
                return HandleResult::Consumed;
            }
            if let Some(prefill) = self.dired_prefill(*id) {
                self.inner
                    .state
                    .put_command(CM_COMMAND_PREFILL, Some(Box::new(prefill)));
                return HandleResult::Consumed;
            }
        }
        if let Event::Command { id, data } = event {
            if *id == CM_OK {
                return self.handle_cm_ok(data);
            }
        }
        self.inner.handle(event)
    }
}

impl FileTreeView {
    fn handle_tick(&mut self) -> HandleResult {
        if self.filter_active {
            return HandleResult::Ignored;
        }
        self.refresh_counter += 1;
        if self.watcher.as_mut().is_some_and(|w| w.has_changes()) {
            self.update_colors();
            self.inner.data.refresh();
            self.inner.mark_dirty();
            self.refresh_counter = 0;
        }
        if self.refresh_counter >= 60 {
            self.refresh_counter = 0;
            self.inner.data.refresh();
            self.inner.mark_dirty();
            self.update_colors();
        }
        HandleResult::Ignored
    }

    fn handle_filter_key(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        match key.code {
            KeyCode::Char('/') if !self.filter_active => {
                self.filter_active = true;
                self.inner.data.ensure_all_loaded();
                self.inner.mark_dirty();
                Some(HandleResult::Consumed)
            }
            KeyCode::Esc if self.filter_active => {
                self.clear_filter();
                Some(HandleResult::Consumed)
            }
            KeyCode::Backspace if self.filter_active => {
                let mut f = self.inner.data.filter().to_string();
                f.pop();
                if f.is_empty() {
                    self.clear_filter();
                } else {
                    self.inner.data.set_filter(&f);
                    self.inner.cursor = 0;
                    self.inner.mark_dirty();
                }
                Some(HandleResult::Consumed)
            }
            KeyCode::Char(c) if self.filter_active => {
                let mut f = self.inner.data.filter().to_string();
                f.push(c);
                self.inner.data.set_filter(&f);
                self.inner.cursor = 0;
                self.inner.mark_dirty();
                Some(HandleResult::Consumed)
            }
            _ => None,
        }
    }

    fn handle_cm_ok(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(&node_id) = boxed.downcast_ref::<usize>() {
                let path = self.inner.data.path(node_id).to_path_buf();
                if !path.is_dir() {
                    let cmd = if self.last_key_was_right {
                        CM_OPEN_FILE_FOCUS
                    } else {
                        CM_OPEN_FILE
                    };
                    self.inner
                        .state
                        .put_command(cmd, Some(Box::new(OpenFileRequest::new(path))));
                }
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::FileTreeView;
    use crate::commands::CM_OPEN_FILE;
    use txv_core::prelude::*;

    #[test]
    fn right_arrow_on_expanded_dir_does_not_open_file() {
        // Create a temp dir with a subdirectory
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("file.txt"), "hello").unwrap();

        let sink = EventSink::new();
        let mut view = FileTreeView::new(tmp.path().to_path_buf(), None);
        view.set_bounds(Rect::new(0, 0, 40, 10));
        view.set_sink(sink.clone());

        // First Right arrow expands the directory
        let right = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyMod::default(),
        });
        view.handle(&right);
        // Should not emit CM_OPEN_FILE (just expanded)
        let events = sink.drain();
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_OPEN_FILE)));

        // Second Right arrow on already-expanded dir should NOT emit CM_OPEN_FILE
        view.handle(&right);
        let events = sink.drain();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_OPEN_FILE)),
            "CM_OPEN_FILE should not be emitted for directories"
        );
    }
}
