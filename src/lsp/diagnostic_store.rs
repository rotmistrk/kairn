//! DiagnosticStore — maps file URI to diagnostics list.

use std::collections::HashMap;

use super::diagnostics::Diagnostic;

/// Diagnostics storage — maps file URI to diagnostics list.
#[derive(Default)]
pub struct DiagnosticStore {
    store: HashMap<String, Vec<Diagnostic>>,
}

impl DiagnosticStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update diagnostics for a file (replaces all previous).
    pub fn set(&mut self, uri: &str, diagnostics: Vec<Diagnostic>) {
        self.store.insert(uri.to_string(), diagnostics);
    }

    /// Get diagnostics for a file.
    pub fn get(&self, uri: &str) -> &[Diagnostic] {
        self.store.get(uri).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get the first diagnostic on a given line.
    pub fn at_line(&self, uri: &str, line: usize) -> Option<&Diagnostic> {
        self.get(uri).iter().find(|d| d.line == line)
    }

    /// Clear all diagnostics.
    pub fn clear(&mut self) {
        self.store.clear();
    }
}
