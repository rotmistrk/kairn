//! Key handling and command dispatch for GitChangesView.

use txv_core::prelude::*;

use crate::commands::*;

use super::GitChangesView;

impl GitChangesView {
    pub(super) fn handle_cm_ok(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        let Some(boxed) = data.as_ref() else {
            return HandleResult::Ignored;
        };
        let Some(&node_id) = boxed.downcast_ref::<usize>() else {
            return HandleResult::Ignored;
        };
        let path = self.inner.data_mut().file_path(node_id).map(|p| p.to_path_buf());
        let Some(path) = path else {
            return HandleResult::Ignored;
        };
        let untracked = self.inner.data_mut().is_untracked(node_id);
        let cmd = if self.last_key_was_right {
            CM_OPEN_FILE_FOCUS
        } else {
            CM_OPEN_FILE
        };
        let req = if untracked {
            OpenFileRequest::new(path.clone())
        } else if let Some(base) = self.find_base_for_path(&path) {
            OpenFileRequest::with_diff_base(path, base)
        } else {
            OpenFileRequest::with_diff(path)
        };
        self.inner.state_mut().put_command(cmd, Some(Box::new(req)));
        HandleResult::Consumed
    }

    pub(super) fn handle_git_key(&mut self, key: &KeyEvent) -> Option<HandleResult> {
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

    pub(super) fn find_base_for_path(&self, path: &std::path::Path) -> Option<String> {
        self.roots
            .iter()
            .filter(|r| path.starts_with(r))
            .max_by_key(|r| r.as_os_str().len())
            .and_then(|r| self.diff_base.get(r).cloned())
    }
}
