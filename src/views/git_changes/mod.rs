//! GitChangesView — non-closeable tab showing changed files grouped by status.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_widgets::TreeView;

use crate::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use crate::git_watcher::WatchHandle;

pub use self::data::GitChangesData;
mod data;

/// The git changes view — wraps TreeView<GitChangesData>.
pub struct GitChangesView {
    inner: TreeView<GitChangesData>,
    watcher: Option<WatchHandle>,
    root: PathBuf,
}

impl GitChangesView {
    pub fn new(root: PathBuf, watcher: Option<WatchHandle>) -> Self {
        let data = GitChangesData::new(&root);
        Self {
            inner: TreeView::new(data),
            watcher,
            root,
        }
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
        let result = self.inner.handle(event, queue);
        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, data } = &ev {
                if *id == CM_OK {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(&node_id) = boxed.downcast_ref::<usize>() {
                            if let Some(path) = self.inner.data.file_path(node_id) {
                                let req = OpenFileRequest::new(path.to_path_buf());
                                queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
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
