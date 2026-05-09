//! Completion infrastructure — trait for providing completions to input widgets.

/// A single completion candidate.
pub struct Completion {
    /// Text to insert at cursor.
    pub text: String,
    /// Display string (may differ from text, e.g. include description).
    pub display: String,
    /// Kind label: "command", "file", "option", etc.
    pub kind: &'static str,
}

/// Trait for providing completions. Implemented by application-level completers.
pub trait Completer: Send {
    /// Return completions for the given input at the given cursor position.
    fn complete(&self, input: &str, cursor: usize) -> Vec<Completion>;
}
