//! Transient pending operations state.

use std::path::PathBuf;
use std::sync::Arc;

use crate::commands::ConfirmContext;
use crate::eviction::PendingTab;
use crate::grep::GrepState;

/// Transient async operations awaiting completion.
pub(crate) struct PendingOps {
    grep_pending: Option<(String, Arc<GrepState>, PathBuf)>,
    pending_tab: Option<PendingTab>,
    confirm_context: Option<ConfirmContext>,
    todo_note_path: Option<Vec<usize>>,
}

impl PendingOps {
    pub(crate) fn new() -> Self {
        Self {
            grep_pending: None,
            pending_tab: None,
            confirm_context: None,
            todo_note_path: None,
        }
    }

    pub(crate) fn grep_pending(&self) -> &Option<(String, Arc<GrepState>, PathBuf)> {
        &self.grep_pending
    }

    pub(crate) fn set_grep_pending(&mut self, val: Option<(String, Arc<GrepState>, PathBuf)>) {
        self.grep_pending = val;
    }

    pub(crate) fn take_grep_pending(&mut self) -> Option<(String, Arc<GrepState>, PathBuf)> {
        self.grep_pending.take()
    }

    pub(crate) fn pending_tab(&self) -> &Option<PendingTab> {
        &self.pending_tab
    }

    pub(crate) fn set_pending_tab(&mut self, val: Option<PendingTab>) {
        self.pending_tab = val;
    }

    pub(crate) fn take_pending_tab(&mut self) -> Option<PendingTab> {
        self.pending_tab.take()
    }

    pub(crate) fn confirm_context(&self) -> &Option<ConfirmContext> {
        &self.confirm_context
    }

    pub(crate) fn set_confirm_context(&mut self, val: Option<ConfirmContext>) {
        self.confirm_context = val;
    }

    pub(crate) fn take_confirm_context(&mut self) -> Option<ConfirmContext> {
        self.confirm_context.take()
    }

    pub(crate) fn todo_note_path(&self) -> &Option<Vec<usize>> {
        &self.todo_note_path
    }

    pub(crate) fn set_todo_note_path(&mut self, val: Option<Vec<usize>>) {
        self.todo_note_path = val;
    }
}
