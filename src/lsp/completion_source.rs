//! DropdownSource for LSP completion items.

use txv_widgets::dropdown_source::DropdownSource;

use super::requests::{CompletionItem, CompletionKind};

/// DropdownSource wrapping LSP completion items.
pub struct LspCompletionSource {
    items: Vec<CompletionItem>,
}

impl LspCompletionSource {
    pub fn new(items: Vec<CompletionItem>) -> Self {
        Self { items }
    }

    pub fn text_at(&self, idx: usize) -> Option<&str> {
        self.items
            .get(idx)
            .map(|i| i.insert_text.as_deref().unwrap_or(&i.label))
    }

    pub fn item_at(&self, idx: usize) -> Option<&CompletionItem> {
        self.items.get(idx)
    }
}

impl DropdownSource for LspCompletionSource {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, idx: usize) -> &str {
        let Some(item) = self.items.get(idx) else {
            return "";
        };
        match item.kind {
            CompletionKind::Function | CompletionKind::Method => &item.label,
            CompletionKind::Other => &item.label,
        }
    }

    fn secondary(&self, idx: usize) -> &str {
        self.items.get(idx).and_then(|i| i.detail.as_deref()).unwrap_or("")
    }
}
