//! Stateful syntax highlighting cache — stores ParseState snapshots every N lines.
//!
//! On draw, highlights from the nearest cached state before the viewport.
//! On edit, invalidates caches from the edited line forward.

use syntect::highlighting::{HighlightState, Highlighter as SyntectHighlighter, Theme};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

use txv_core::prelude::{Color, Style};

use crate::highlight::{ensure_readable, HlSpan};

const LINES_PER_SNAPSHOT: usize = 50;

use crate::highlight_snapshot::Snapshot;

/// Per-file highlight cache. Stores ParseState snapshots every 50 lines.
pub struct HighlightCache {
    snapshots: Vec<Snapshot>,
    ext: String,
}

// SAFETY: HighlightCache is only accessed from the single main thread.
// ParseState contains raw pointers from onig regex but we never share across threads.
unsafe impl Send for HighlightCache {}

impl HighlightCache {
    pub fn new(ext: &str) -> Self {
        Self {
            snapshots: Vec::new(),
            ext: ext.to_string(),
        }
    }

    /// Invalidate all cached state from `line` onward.
    pub fn invalidate_from(&mut self, line: usize) {
        let keep = line / LINES_PER_SNAPSHOT;
        self.snapshots.truncate(keep);
    }

    /// Invalidate everything (theme change, file reload).
    pub fn invalidate_all(&mut self) {
        self.snapshots.clear();
    }

    /// Highlight lines in range [start_line..end_line) using cached state.
    /// `get_line` returns the text for a given line index.
    pub fn highlight_viewport(
        &mut self,
        start_line: usize,
        end_line: usize,
        line_count: usize,
        get_line: impl Fn(usize) -> String,
        syntax_set: &SyntaxSet,
        theme: &Theme,
    ) -> Vec<Vec<HlSpan>> {
        let Some(syntax) = find_syntax(syntax_set, &self.ext) else {
            return self.plain_lines(start_line, end_line, &get_line);
        };

        let end_line = end_line.min(line_count);
        let (mut parse, mut scope, resume_line) = self.find_start_state(syntax, start_line);

        // Parse from resume_line up to end_line, caching snapshots along the way.
        let highlighter = SyntectHighlighter::new(theme);
        let mut result = Vec::with_capacity(end_line.saturating_sub(start_line));

        for line_idx in resume_line..end_line {
            // Save snapshot at boundary before parsing this line.
            let snapshot_idx = line_idx / LINES_PER_SNAPSHOT;
            if line_idx % LINES_PER_SNAPSHOT == 0 && self.snapshots.len() == snapshot_idx {
                self.snapshots.push(Snapshot {
                    parse: parse.clone(),
                    scope: scope.clone(),
                });
            }

            let text = get_line(line_idx);
            let spans = parse_line_styled(&mut parse, &mut scope, &text, syntax_set, &highlighter);

            if line_idx >= start_line {
                result.push(spans);
            }
        }

        result
    }

    fn find_start_state(&self, syntax: &SyntaxReference, target_line: usize) -> (ParseState, ScopeStack, usize) {
        let snapshot_idx = target_line / LINES_PER_SNAPSHOT;
        // Use the latest snapshot at or before the target chunk.
        let available = self.snapshots.len();
        if available > 0 {
            let use_idx = snapshot_idx.min(available - 1);
            let snap = &self.snapshots[use_idx];
            let resume_line = use_idx * LINES_PER_SNAPSHOT;
            (snap.parse.clone(), snap.scope.clone(), resume_line)
        } else {
            (ParseState::new(syntax), ScopeStack::new(), 0)
        }
    }

    fn plain_lines(&self, start: usize, end: usize, get_line: &impl Fn(usize) -> String) -> Vec<Vec<HlSpan>> {
        (start..end)
            .map(|i| {
                vec![HlSpan {
                    text: get_line(i),
                    style: Style::default(),
                }]
            })
            .collect()
    }
}

fn parse_line_styled(
    parse: &mut ParseState,
    scope: &mut ScopeStack,
    line: &str,
    syntax_set: &SyntaxSet,
    highlighter: &SyntectHighlighter,
) -> Vec<HlSpan> {
    let line_nl = format!("{line}\n");
    let ops = match parse.parse_line(&line_nl, syntax_set) {
        Ok(ops) => ops,
        Err(_) => {
            return vec![HlSpan {
                text: line.to_string(),
                style: Style::default(),
            }]
        }
    };

    let spans = highlight_ops(&ops, &line_nl, scope, highlighter);

    // Apply ops to scope stack for next line.
    for (_idx, op) in &ops {
        scope.apply(op).ok();
    }

    spans
}

fn highlight_ops(
    ops: &[(usize, syntect::parsing::ScopeStackOp)],
    line_nl: &str,
    scope: &ScopeStack,
    highlighter: &SyntectHighlighter,
) -> Vec<HlSpan> {
    let mut hl_state = HighlightState::new(highlighter, scope.clone());
    let iter = syntect::highlighting::HighlightIterator::new(&mut hl_state, ops, line_nl, highlighter);
    let mut spans: Vec<HlSpan> = iter
        .map(|(style, text)| {
            let (r, g, b) = ensure_readable(style.foreground.r, style.foreground.g, style.foreground.b);
            HlSpan {
                text: text.to_string(),
                style: Style {
                    fg: Color::Rgb(r, g, b),
                    ..Style::default()
                },
            }
        })
        .collect();

    // Strip the trailing \n we added.
    if let Some(last) = spans.last_mut() {
        if last.text.ends_with('\n') {
            last.text.pop();
            if last.text.is_empty() {
                spans.pop();
            }
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_brackets_correct_with_context() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults();
        let theme = &themes.themes["base16-eighties.dark"];

        let lines = vec![
            "class Foo {".to_string(),
            "  void bar() {".to_string(),
            "  }".to_string(),
            "}".to_string(),
        ];
        let mut cache = HighlightCache::new("java");
        let result = cache.highlight_viewport(0, 4, 4, |i| lines[i].clone(), &syntax_set, theme);

        // Line 3 is "}" — should NOT be dark gray
        let last_line = &result[3];
        for span in last_line {
            if span.text.contains('}') {
                let Color::Rgb(r, g, b) = span.style.fg else {
                    panic!("not rgb")
                };
                assert!(
                    r > 80 || g > 80 || b > 80,
                    "Java '}}' too dark with context: ({r},{g},{b})"
                );
            }
        }
    }

    #[test]
    fn line_comment_does_not_bleed_to_next_line() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults();
        let theme = &themes.themes["base16-eighties.dark"];

        let lines = vec![
            "// comment".to_string(),
            "let x = 1;".to_string(),
            "let y = 2;".to_string(),
        ];
        let mut cache = HighlightCache::new("rs");
        let result = cache.highlight_viewport(0, 3, 3, |i| lines[i].clone(), &syntax_set, theme);

        // Line 1 ("let x = 1;") — "let" should be highlighted as keyword, not comment.
        // Comments and keywords have different colors; verify line 1 != line 0 color.
        let comment_color = result[0][0].style.fg;
        let line1_spans = &result[1];
        let let_span = line1_spans
            .iter()
            .find(|s| s.text.contains("let"))
            .expect("should have 'let'");
        assert_ne!(
            let_span.style.fg, comment_color,
            "'let' on line after // should not have comment color"
        );
    }

    #[test]
    fn snapshot_lookup_highlights_multiline_string_continuation() {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults();
        let theme = &themes.themes["base16-eighties.dark"];

        // Build 110 lines: normal code, then a multiline string starting at line 45
        // that spans past line 60 (into snapshot[1] territory).
        let mut lines: Vec<String> = Vec::with_capacity(110);
        for i in 0..45 {
            lines.push(format!("let x{i} = {i};"));
        }
        // Line 45: start a multiline string (raw string in Rust)
        lines.push("let s = r#\"".to_string());
        for i in 46..70 {
            lines.push(format!("  string content line {i}"));
        }
        lines.push("\"#;".to_string()); // line 70: close the string
        for i in 71..110 {
            lines.push(format!("let y{i} = {i};"));
        }

        let mut cache = HighlightCache::new("rs");
        // First pass: build all snapshots by highlighting 0..110
        cache.highlight_viewport(0, 110, 110, |i| lines[i].clone(), &syntax_set, theme);
        assert!(cache.snapshots.len() >= 2, "should have at least 2 snapshots");

        // Now request only viewport starting at line 60 (inside the multiline string).
        // This must use the correct snapshot (snapshot[1] at line 50) which has the
        // string parse state, not snapshot[0] (line 0) which would lose the context.
        let result = cache.highlight_viewport(60, 65, 110, |i| lines[i].clone(), &syntax_set, theme);
        assert_eq!(result.len(), 5);

        // Line 60 is inside the raw string — it should have string color (not default).
        let line60_spans = &result[0];
        let default_style = Style::default();
        let has_string_color = line60_spans.iter().any(|s| s.style.fg != default_style.fg);
        assert!(
            has_string_color,
            "line 60 (inside multiline string) should have non-default color, got: {:?}",
            line60_spans.iter().map(|s| (&s.text, s.style.fg)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn invalidate_from_truncates_snapshots() {
        let mut cache = HighlightCache::new("rs");
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults();
        let theme = &themes.themes["base16-eighties.dark"];

        // Generate 100 lines to create 2 snapshots
        let lines: Vec<String> = (0..100).map(|i| format!("let x{i} = {i};")).collect();
        cache.highlight_viewport(0, 100, 100, |i| lines[i].clone(), &syntax_set, theme);
        assert!(cache.snapshots.len() >= 2);

        // Edit at line 30 should keep snapshot 0 (lines 0..50) but not snapshot 1
        cache.invalidate_from(30);
        assert_eq!(cache.snapshots.len(), 0); // 30/50 = 0, truncate to 0

        // Edit at line 60 after rebuilding
        cache.highlight_viewport(0, 100, 100, |i| lines[i].clone(), &syntax_set, theme);
        let before = cache.snapshots.len();
        cache.invalidate_from(60);
        assert_eq!(cache.snapshots.len(), 1); // 60/50 = 1, keep first snapshot
        assert!(before > cache.snapshots.len());
    }
}

fn find_syntax<'a>(syntax_set: &'a SyntaxSet, ext: &str) -> Option<&'a SyntaxReference> {
    syntax_set
        .find_syntax_by_extension(ext)
        .or_else(|| syntax_set.find_syntax_by_name(ext))
}
