//! Confirmation context enum.

/// Context for which confirmation is active.
#[derive(Debug, Clone)]
pub enum ConfirmContext {
    EditorClose(String),
    FileReload(String),
    Quit,
    TodoDelete,
    TodoCrypto,
    CsvDeleteRow,
}
