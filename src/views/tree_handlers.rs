//! FileTreeView event handlers — save, tick, filter, open.

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::prelude::*;

use crate::commands::{OpenFileRequest, CM_OPEN_FILE, CM_OPEN_FILE_FOCUS};
use crate::views::tree::FileTreeView;

impl FileTreeView {
    pub(super) fn handle_save(&mut self) -> HandleResult {
        self.notify_save();
        self.inner.data.refresh();
        self.inner.mark_dirty();
        self.request_colors();
        HandleResult::Ignored
    }

    pub(super) fn handle_tick(&mut self) -> HandleResult {
        if self.filter_active {
            self.apply_pending_colors();
            return HandleResult::Ignored;
        }
        self.apply_pending_colors();
        self.refresh_counter += 1;
        if self.watcher.as_mut().is_some_and(|w| w.has_changes()) {
            self.request_colors();
            self.inner.data.refresh();
            self.inner.mark_dirty();
            self.refresh_counter = 0;
        }
        if self.refresh_counter >= 60 {
            self.refresh_counter = 0;
            self.inner.data.refresh();
            self.inner.mark_dirty();
            self.request_colors();
        }
        HandleResult::Ignored
    }

    pub(super) fn handle_filter_key(&mut self, key: &KeyEvent) -> Option<HandleResult> {
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

    pub(super) fn handle_cm_ok(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
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
