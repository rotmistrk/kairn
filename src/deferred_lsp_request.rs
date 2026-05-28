//! Deferred LSP request — queued while server is initializing.

/// A deferred LSP request waiting for server initialization.
pub struct DeferredLspRequest {
    pub(crate) command: txv_core::prelude::CommandId,
    pub(crate) data: Box<dyn std::any::Any + Send>,
    pub(crate) language: String,
    pub(crate) created: std::time::Instant,
}
