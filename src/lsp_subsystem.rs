//! LSP subsystem state: registry, pending requests, connection state, languages.

use txv_core::view::EventSink;

use crate::completer::LspLanguageList;
use crate::lsp::pending::PendingRequests;
use crate::lsp::registry::LspRegistry;
use crate::lsp_state::LspState;

/// LSP subsystem: registry, pending requests, state, languages.
pub(crate) struct LspSubsystem {
    registry: LspRegistry,
    pending: PendingRequests,
    state: LspState,
    languages: LspLanguageList,
}

impl LspSubsystem {
    pub(crate) fn new(pending: PendingRequests, languages: LspLanguageList) -> Self {
        Self {
            registry: LspRegistry::new(),
            pending,
            state: LspState::new(),
            languages,
        }
    }

    pub(crate) fn registry(&self) -> &LspRegistry {
        &self.registry
    }

    pub(crate) fn registry_mut(&mut self) -> &mut LspRegistry {
        &mut self.registry
    }

    pub(crate) fn pending(&self) -> &PendingRequests {
        &self.pending
    }

    pub(crate) fn pending_mut(&mut self) -> &mut PendingRequests {
        &mut self.pending
    }

    pub(crate) fn state(&self) -> &LspState {
        &self.state
    }

    pub(crate) fn state_mut(&mut self) -> &mut LspState {
        &mut self.state
    }

    pub(crate) fn languages(&self) -> &LspLanguageList {
        &self.languages
    }

    /// Split borrow: mutable registry and mutable state simultaneously.
    pub(crate) fn registry_and_state(&mut self) -> (&mut LspRegistry, &mut LspState) {
        (&mut self.registry, &mut self.state)
    }

    /// Split borrow: mutable registry and mutable pending simultaneously.
    pub(crate) fn registry_and_pending(&mut self) -> (&mut LspRegistry, &mut PendingRequests) {
        (&mut self.registry, &mut self.pending)
    }

    /// Remove timed-out pending requests (needs both pending and registry).
    pub(crate) fn remove_timed_out(&mut self, sink: &EventSink) {
        self.pending.remove_timed_out(sink, &self.registry);
    }
}
