//! Cached parse state snapshot for syntax highlighting.

use syntect::parsing::{ParseState, ScopeStack};

/// Cached parse state at a specific line boundary.
#[derive(Clone)]
pub(crate) struct Snapshot {
    pub(crate) parse: ParseState,
    pub(crate) scope: ScopeStack,
}
