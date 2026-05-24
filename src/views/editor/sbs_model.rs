//! Side-by-side diff model types and split logic.

use super::diff_model::DiffLine;

#[derive(Clone, Debug, PartialEq)]
pub enum SbsLine {
    /// Content line from this side's file. `changed` = true if it's a diff hunk line.
    Content {
        line_no: usize,
        text: String,
        changed: bool,
    },
    /// Gap — the other side has a line here, this side doesn't.
    Gap,
    /// Folded context (same count on both sides).
    Folded { count: usize },
}

/// Side-by-side diff state for one pane.
pub struct SbsDiffState {
    pub lines: Vec<SbsLine>,
    pub scroll: usize,
    pub cursor: usize,
    pub base_ref: String,
    pub is_left: bool,
}

/// Split unified diff lines into left (base) and right (current) streams.
pub fn split_for_side_by_side(
    unified: &[DiffLine],
    base_text: &str,
    current_text: &str,
) -> (Vec<SbsLine>, Vec<SbsLine>) {
    let base_lines: Vec<&str> = base_text.lines().collect();
    let current_lines: Vec<&str> = current_text.lines().collect();
    let mut left = Vec::new();
    let mut right = Vec::new();

    for dl in unified {
        match dl {
            DiffLine::Context { buf_line, base_line } => {
                let lt = base_lines.get(*base_line).unwrap_or(&"").to_string();
                let rt = current_lines.get(*buf_line).unwrap_or(&"").to_string();
                left.push(SbsLine::Content {
                    line_no: *base_line,
                    text: lt,
                    changed: false,
                });
                right.push(SbsLine::Content {
                    line_no: *buf_line,
                    text: rt,
                    changed: false,
                });
            }
            DiffLine::Deleted { base_line, text } => {
                left.push(SbsLine::Content {
                    line_no: *base_line,
                    text: text.clone(),
                    changed: true,
                });
                right.push(SbsLine::Gap);
            }
            DiffLine::Added { buf_line } => {
                let rt = current_lines.get(*buf_line).unwrap_or(&"").to_string();
                left.push(SbsLine::Gap);
                right.push(SbsLine::Content {
                    line_no: *buf_line,
                    text: rt,
                    changed: true,
                });
            }
            DiffLine::Folded { count } => {
                left.push(SbsLine::Folded { count: *count });
                right.push(SbsLine::Folded { count: *count });
            }
        }
    }
    (left, right)
}
