//! Grouped LSP-related state fields extracted from AppState.

use std::collections::{HashMap, HashSet};

use crate::deferred_lsp_request::DeferredLspRequest;
use crate::lsp::progress::LspStatusTracker;

pub(crate) struct LspState {
    doc_versions: HashMap<String, i64>,
    opened_files: HashSet<String>,
    deferred: Vec<DeferredLspRequest>,
    status: LspStatusTracker,
}

impl LspState {
    pub(crate) fn new() -> Self {
        Self {
            doc_versions: HashMap::new(),
            opened_files: HashSet::new(),
            deferred: Vec::new(),
            status: LspStatusTracker::new(),
        }
    }

    pub(crate) fn doc_versions(&self) -> &HashMap<String, i64> {
        &self.doc_versions
    }

    pub(crate) fn doc_versions_mut(&mut self) -> &mut HashMap<String, i64> {
        &mut self.doc_versions
    }

    pub(crate) fn opened_files(&self) -> &HashSet<String> {
        &self.opened_files
    }

    pub(crate) fn opened_files_mut(&mut self) -> &mut HashSet<String> {
        &mut self.opened_files
    }

    pub(crate) fn deferred(&self) -> &[DeferredLspRequest] {
        &self.deferred
    }

    pub(crate) fn deferred_mut(&mut self) -> &mut Vec<DeferredLspRequest> {
        &mut self.deferred
    }

    pub(crate) fn status(&self) -> &LspStatusTracker {
        &self.status
    }

    pub(crate) fn status_mut(&mut self) -> &mut LspStatusTracker {
        &mut self.status
    }
}
