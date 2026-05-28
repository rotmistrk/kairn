//! Pending LSP request tracking — maps request IDs to expected response types.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use txv_core::prelude::*;

/// Tracks pending LSP requests so responses can be routed.
pub struct PendingRequests {
    map: HashMap<u64, (PendingKind, String, Instant)>,
    pub(crate) timeout_secs: u64,
}

impl Default for PendingRequests {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PendingKind {
    GotoDefinition,
    GotoShow,
    FindReferences { symbol: String },
    Hover,
    Completion,
    SignatureHelp,
    Rename,
    CodeAction,
    JdtClassContents { line: u32, character: u32 },
}

impl PendingRequests {
    pub(crate) fn insert(&mut self, id: u64, kind: PendingKind) {
        self.insert_with_lang(id, kind, "");
    }

    pub(crate) fn insert_with_lang(&mut self, id: u64, kind: PendingKind, lang: &str) {
        self.map.insert(id, (kind, lang.to_string(), Instant::now()));
    }

    pub(crate) fn take(&mut self, id: u64) -> Option<PendingKind> {
        self.map.remove(&id).map(|(k, _, _)| k)
    }

    pub(crate) fn remove_timed_out(&mut self, sink: &EventSink, registry: &super::registry::LspRegistry) {
        let global_timeout = self.timeout_secs;
        let expired: Vec<u64> = self
            .map
            .iter()
            .filter(|(_, (_, lang, t))| {
                let secs = registry.timeout(lang).unwrap_or(global_timeout);
                t.elapsed() > Duration::from_secs(secs)
            })
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            if let Some((kind, lang, _)) = self.map.remove(&id) {
                let secs = registry.timeout(&lang).unwrap_or(global_timeout);
                let label = friendly_kind_label(&kind);
                let msg = format!("{label}: no response after {secs}s");
                log::warn!("LSP timeout: {msg}");
                sink.push_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::error("lsp", msg))),
                );
            }
        }
    }
}

fn friendly_kind_label(kind: &PendingKind) -> &'static str {
    match kind {
        PendingKind::GotoDefinition => "Go to definition",
        PendingKind::GotoShow => "Go to definition",
        PendingKind::FindReferences { .. } => "Find references",
        PendingKind::Hover => "Hover",
        PendingKind::Completion => "Completion",
        PendingKind::SignatureHelp => "Signature help",
        PendingKind::Rename => "Rename",
        PendingKind::CodeAction => "Code action",
        PendingKind::JdtClassContents { .. } => "Class contents",
    }
}

pub(crate) use super::jdt_request::JdtRequest;
