//! Shared editor state: register, clipboard, histories, linked scroll.

use std::sync::Arc;

use txv_core::clipboard_ring::ClipboardHandle;
use txv_core::shared_history::SharedHistory;

use crate::shared_register::RegisterHandle;

/// Shared state across all editor instances.
pub(crate) struct EditorShared {
    shared_register: RegisterHandle,
    clipboard: ClipboardHandle,
    command_history: SharedHistory,
    search_history: SharedHistory,
    linked_scroll: bool,
}

impl EditorShared {
    pub(crate) fn new(
        clipboard: ClipboardHandle,
        command_history: SharedHistory,
        search_history: SharedHistory,
    ) -> Self {
        Self {
            shared_register: Arc::default(),
            clipboard,
            command_history,
            search_history,
            linked_scroll: false,
        }
    }

    pub(crate) fn shared_register(&self) -> &RegisterHandle {
        &self.shared_register
    }

    pub(crate) fn clipboard(&self) -> &ClipboardHandle {
        &self.clipboard
    }

    pub(crate) fn clipboard_mut(&mut self) -> &mut ClipboardHandle {
        &mut self.clipboard
    }

    pub(crate) fn command_history(&self) -> &SharedHistory {
        &self.command_history
    }

    pub(crate) fn search_history(&self) -> &SharedHistory {
        &self.search_history
    }

    pub(crate) fn linked_scroll(&self) -> bool {
        self.linked_scroll
    }

    pub(crate) fn set_linked_scroll(&mut self, on: bool) {
        self.linked_scroll = on;
    }
}
