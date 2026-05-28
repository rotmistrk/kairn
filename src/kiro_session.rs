//! Metadata for a single kiro tab.

/// Metadata for a single kiro tab.
#[derive(Debug, Clone)]
pub struct KiroSession {
    pub(crate) display_name: String,
    pub(crate) session_id: Option<String>,
}
