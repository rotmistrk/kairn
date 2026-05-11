//! Diff mode data model — virtual line list, navigation, folding.

/// A single line in the diff virtual view.
#[derive(Clone, Debug, PartialEq)]
pub enum DiffLine {
    /// Unchanged line — exists in both base and current.
    Context { buf_line: usize, base_line: usize },
    /// Added line — only in current buffer.
    Added { buf_line: usize },
    /// Deleted line — only in base (virtual, not in buffer).
    Deleted { text: String, base_line: usize },
    /// Folded context — N hidden unchanged lines.
    Folded { count: usize },
}

/// Diff mode state stored in EditorView.
pub struct DiffState {
    pub lines: Vec<DiffLine>,
    pub scroll: usize,
    pub cursor: usize,
    pub base_ref: String,
    pub context_lines: usize,
    pub ignore_ws: bool,
}

/// Options parsed from :diff args.
pub struct DiffOpts {
    pub base: String,
    pub context: usize,
    pub ignore_ws: bool,
}

pub fn parse_diff_args(args: &str) -> DiffOpts {
    let mut base = "HEAD".to_string();
    let mut context = usize::MAX;
    let mut ignore_ws = false;
    for arg in args.split_whitespace() {
        if arg == "-w" {
            ignore_ws = true;
        } else if let Some(n) = arg.strip_prefix("-U") {
            context = n.parse().unwrap_or(3);
        } else if !arg.starts_with('-') {
            base = arg.to_string();
        }
    }
    DiffOpts {
        base,
        context,
        ignore_ws,
    }
}

pub fn is_change(line: &DiffLine) -> bool {
    matches!(line, DiffLine::Added { .. } | DiffLine::Deleted { .. })
}

impl DiffState {
    /// Buffer line at current cursor (for jump-and-exit).
    pub fn cursor_buf_line(&self) -> usize {
        for i in (0..=self.cursor).rev() {
            match &self.lines[i] {
                DiffLine::Context { buf_line, .. } | DiffLine::Added { buf_line } => {
                    return *buf_line;
                }
                _ => {}
            }
        }
        for line in &self.lines[self.cursor..] {
            match line {
                DiffLine::Context { buf_line, .. } | DiffLine::Added { buf_line } => {
                    return *buf_line;
                }
                _ => {}
            }
        }
        0
    }

    /// Next hunk (first Added/Deleted after current hunk).
    pub fn next_hunk(&self) -> Option<usize> {
        let mut i = self.cursor + 1;
        // If cursor is on a change, skip rest of current hunk
        if is_change(&self.lines[self.cursor]) {
            while i < self.lines.len() && is_change(&self.lines[i]) {
                i += 1;
            }
        }
        // Find next change
        while i < self.lines.len() {
            if is_change(&self.lines[i]) {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Previous hunk start.
    pub fn prev_hunk(&self) -> Option<usize> {
        if self.cursor == 0 {
            return None;
        }
        // Find the start of the region before the current hunk
        let mut i = self.cursor;
        // Skip current hunk backward (including cursor position)
        while i > 0 && is_change(&self.lines[i]) {
            i -= 1;
        }
        // Now i is on a non-change line (or 0). Search backward for a change.
        loop {
            if is_change(&self.lines[i]) {
                while i > 0 && is_change(&self.lines[i - 1]) {
                    i -= 1;
                }
                return Some(i);
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        None
    }

    /// Ensure cursor is visible within scroll window.
    pub fn ensure_visible(&mut self, height: usize) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + height {
            self.scroll = self.cursor.saturating_sub(height - 1);
        }
    }
}

/// Build virtual diff line list from base and current content.
pub fn build_diff_lines(base: &str, current: &str, opts: &DiffOpts) -> Vec<DiffLine> {
    use similar::{ChangeTag, TextDiff};

    let (base_cmp, current_cmp) = if opts.ignore_ws {
        (normalize_ws(base), normalize_ws(current))
    } else {
        (base.to_string(), current.to_string())
    };

    let diff = TextDiff::from_lines(&base_cmp, &current_cmp);
    let base_lines: Vec<&str> = base.lines().collect();

    let mut full: Vec<DiffLine> = Vec::new();
    let mut base_idx: usize = 0;
    let mut buf_idx: usize = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                full.push(DiffLine::Context {
                    buf_line: buf_idx,
                    base_line: base_idx,
                });
                base_idx += 1;
                buf_idx += 1;
            }
            ChangeTag::Insert => {
                full.push(DiffLine::Added { buf_line: buf_idx });
                buf_idx += 1;
            }
            ChangeTag::Delete => {
                let text = base_lines.get(base_idx).unwrap_or(&"").to_string();
                full.push(DiffLine::Deleted {
                    text,
                    base_line: base_idx,
                });
                base_idx += 1;
            }
        }
    }

    if opts.context == usize::MAX {
        return full;
    }
    fold_context(&full, opts.context)
}

fn fold_context(lines: &[DiffLine], ctx: usize) -> Vec<DiffLine> {
    let len = lines.len();
    let mut keep = vec![false; len];
    for (i, line) in lines.iter().enumerate() {
        if is_change(line) {
            let start = i.saturating_sub(ctx);
            let end = (i + ctx + 1).min(len);
            for flag in &mut keep[start..end] {
                *flag = true;
            }
        }
    }
    let mut result: Vec<DiffLine> = Vec::new();
    let mut i = 0;
    while i < len {
        if keep[i] {
            result.push(lines[i].clone());
            i += 1;
        } else {
            let start = i;
            while i < len && !keep[i] {
                i += 1;
            }
            result.push(DiffLine::Folded { count: i - start });
        }
    }
    result
}

fn normalize_ws(text: &str) -> String {
    text.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        + if text.ends_with('\n') {
            "\n"
        } else {
            ""
        }
}
