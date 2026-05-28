//! Persisted split panel state.

use serde::{Deserialize, Serialize};

/// Persisted split panel state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitState {
    /// "horizontal" or "vertical"
    pub(crate) direction: String,
    /// Tab indices in the second subpanel (first subpanel has the rest).
    pub(crate) second_tabs: Vec<usize>,
    /// Active tab index in first subpanel.
    pub(crate) active_first: usize,
    /// Active tab index in second subpanel.
    pub(crate) active_second: usize,
    /// Focused subpanel (0 or 1).
    pub(crate) focused: usize,
}
