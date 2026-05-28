//! Persisted kiro tab session.

use serde::{Deserialize, Serialize};

/// Persisted kiro tab session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroSessionState {
    pub(crate) name: String,
    pub(crate) session_id: Option<String>,
}
