//! Payload for CM_DIFF_SPLIT — side-by-side diff.

#[derive(Debug, Clone)]
pub struct DiffSplitRequest {
    pub(crate) base_content: String,
    pub(crate) base_ref: String,
}

impl DiffSplitRequest {
    pub fn new(base_content: impl Into<String>, base_ref: impl Into<String>) -> Self {
        Self {
            base_content: base_content.into(),
            base_ref: base_ref.into(),
        }
    }
    pub fn base_content(&self) -> &str {
        &self.base_content
    }
    pub fn base_ref(&self) -> &str {
        &self.base_ref
    }
}
