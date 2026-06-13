//! Side-by-side diff model — single-view two-column rendering.
//!
//! One EditorView holds both left (base) and right (current) columns.
//! No structural split needed — this is a rendering mode.

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

/// Side-by-side diff state — both columns in one view.
pub struct SbsDiffState {
    pub(crate) left: Vec<SbsLine>,
    pub(crate) right: Vec<SbsLine>,
    pub(crate) scroll: usize,
    pub(crate) cursor: usize,
    pub(crate) base_ref: String,
}

/// Split unified diff lines into left (base) and right (current) streams.
/// Pairs adjacent Deleted+Added lines on the same row for proper SBS display.
pub fn split_for_side_by_side(
    unified: &[DiffLine],
    base_text: &str,
    current_text: &str,
) -> (Vec<SbsLine>, Vec<SbsLine>) {
    let base_lines: Vec<&str> = base_text.lines().collect();
    let current_lines: Vec<&str> = current_text.lines().collect();
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut i = 0;
    while i < unified.len() {
        match &unified[i] {
            DiffLine::Context { buf_line, base_line } => {
                let lt = base_lines.get(*base_line).unwrap_or(&"").to_string();
                let rt = current_lines.get(*buf_line).unwrap_or(&"").to_string();
                left.push(sbs_content(*base_line, lt, false));
                right.push(sbs_content(*buf_line, rt, false));
                i += 1;
            }
            DiffLine::Folded { count } => {
                left.push(SbsLine::Folded { count: *count });
                right.push(SbsLine::Folded { count: *count });
                i += 1;
            }
            DiffLine::Deleted { .. } | DiffLine::Added { .. } => {
                i = pair_hunk_lines(unified, i, &base_lines, &current_lines, &mut left, &mut right);
            }
        }
    }
    (left, right)
}

/// Collect a run of Deleted/Added lines and pair them on same rows.
fn pair_hunk_lines(
    unified: &[DiffLine],
    start: usize,
    _base_lines: &[&str],
    current_lines: &[&str],
    left: &mut Vec<SbsLine>,
    right: &mut Vec<SbsLine>,
) -> usize {
    let mut dels: Vec<SbsLine> = Vec::new();
    let mut adds: Vec<SbsLine> = Vec::new();
    let mut i = start;
    while i < unified.len() {
        match &unified[i] {
            DiffLine::Deleted { base_line, text } => {
                dels.push(sbs_content(*base_line, text.clone(), true));
            }
            DiffLine::Added { buf_line } => {
                let rt = current_lines.get(*buf_line).unwrap_or(&"").to_string();
                adds.push(sbs_content(*buf_line, rt, true));
            }
            _ => break,
        }
        i += 1;
    }
    let max_len = dels.len().max(adds.len());
    for idx in 0..max_len {
        left.push(dels.get(idx).cloned().unwrap_or(SbsLine::Gap));
        right.push(adds.get(idx).cloned().unwrap_or(SbsLine::Gap));
    }
    i
}

fn sbs_content(line_no: usize, text: String, changed: bool) -> SbsLine {
    SbsLine::Content { line_no, text, changed }
}
