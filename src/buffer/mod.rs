pub mod line_index;
pub mod piece_table;
pub mod render;
pub mod undo;

#[allow(unused_imports)]
pub use piece_table::PieceTable;
#[allow(unused_imports)]
pub use piece_table::TextChange;

use serde::{Deserialize, Serialize};

/// What kind of content the buffer holds, for rendering decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BufferKind {
    PlainText,
    SyntaxHighlighted { language: String },
    UnifiedDiff,
    FilePreview { path: String },
}

/// A buffer of content that can be displayed in the main panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBuffer {
    pub title: String,
    pub content: String,
    pub kind: BufferKind,
    pub scroll_offset: u16,
}

impl OutputBuffer {
    pub fn plain(title: String, content: String) -> Self {
        Self {
            title,
            content,
            kind: BufferKind::PlainText,
            scroll_offset: 0,
        }
    }

    pub fn highlighted(title: String, content: String, language: String) -> Self {
        Self {
            title,
            content,
            kind: BufferKind::SyntaxHighlighted { language },
            scroll_offset: 0,
        }
    }

    pub fn diff(title: String, content: String) -> Self {
        Self {
            title,
            content,
            kind: BufferKind::UnifiedDiff,
            scroll_offset: 0,
        }
    }
}
