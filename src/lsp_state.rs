//! Grouped LSP-related state fields extracted from AppState.

use std::collections::{HashMap, HashSet};

use crate::deferred_lsp_request::DeferredLspRequest;
use crate::lsp::progress::LspStatusTracker;

pub(crate) struct LspState {
    pub(crate) doc_versions: HashMap<String, i64>,
    pub(crate) opened_files: HashSet<String>,
    pub(crate) deferred: Vec<DeferredLspRequest>,
    pub(crate) status: LspStatusTracker,
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
}
