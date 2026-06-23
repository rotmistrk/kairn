use std::path::PathBuf;
use std::sync::Arc;

use crate::build::ErrorLocation;
use crate::task_output::TaskOutput;

/// Build-related state (errors, navigation index, pending task).
pub(crate) struct BuildState {
    errors: Vec<ErrorLocation>,
    error_idx: usize,
    pending: Option<(String, Arc<TaskOutput>, PathBuf)>,
}

impl BuildState {
    pub(crate) fn new() -> Self {
        Self {
            errors: Vec::new(),
            error_idx: 0,
            pending: None,
        }
    }

    pub(crate) fn errors(&self) -> &[ErrorLocation] {
        &self.errors
    }

    pub(crate) fn errors_mut(&mut self) -> &mut Vec<ErrorLocation> {
        &mut self.errors
    }

    pub(crate) fn error_idx(&self) -> usize {
        self.error_idx
    }

    pub(crate) fn set_error_idx(&mut self, idx: usize) {
        self.error_idx = idx;
    }

    pub(crate) fn pending(&self) -> &Option<(String, Arc<TaskOutput>, PathBuf)> {
        &self.pending
    }

    pub(crate) fn pending_mut(&mut self) -> &mut Option<(String, Arc<TaskOutput>, PathBuf)> {
        &mut self.pending
    }

    pub(crate) fn take_pending(&mut self) -> Option<(String, Arc<TaskOutput>, PathBuf)> {
        self.pending.take()
    }

    pub(crate) fn set_pending(&mut self, val: Option<(String, Arc<TaskOutput>, PathBuf)>) {
        self.pending = val;
    }
}
