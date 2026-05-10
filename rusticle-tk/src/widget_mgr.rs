//! Widget helpers — data types for script-created widgets.

use txv_core::prelude::*;
use txv_widgets::ListData;

/// Simple string-backed list data for `ListView`.
pub struct StringListData {
    /// The string items.
    pub items: Vec<String>,
}

impl StringListData {
    /// Create from a vec of strings.
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl ListData for StringListData {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, index: usize) -> &str {
        self.items.get(index).map(|s| s.as_str()).unwrap_or("")
    }

    fn style(&self, _index: usize) -> Style {
        Style::default()
    }
}
