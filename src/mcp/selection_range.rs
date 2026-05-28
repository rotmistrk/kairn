use serde::Serialize;

/// Selection range in an editor tab.
#[derive(Debug, Clone, Serialize)]
pub struct SelectionRange {
    pub(crate) start_line: usize,
    pub(crate) start_col: usize,
    pub(crate) end_line: usize,
    pub(crate) end_col: usize,
}
