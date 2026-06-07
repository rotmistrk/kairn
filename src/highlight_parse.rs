//! Syntax highlighting helpers: line parsing and span coloring.

use syntect::highlighting::HighlightIterator;
use syntect::highlighting::{HighlightState, Highlighter as SyntectHighlighter};
use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use txv_core::prelude::*;

use crate::highlight::ensure_readable;
use crate::hl_span::HlSpan;

pub(crate) fn parse_line_styled(
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

pub(crate) fn highlight_ops(
    ops: &[(usize, syntect::parsing::ScopeStackOp)],
    line_nl: &str,
    scope: &ScopeStack,
    highlighter: &SyntectHighlighter,
) -> Vec<HlSpan> {
    let mut hl_state = HighlightState::new(highlighter, scope.clone());
    let iter = HighlightIterator::new(&mut hl_state, ops, line_nl, highlighter);
    let mut spans: Vec<HlSpan> = iter
        .map(|(style, text)| {
            let (r, g, b) = ensure_readable(style.foreground.r, style.foreground.g, style.foreground.b);
            HlSpan {
                text: text.to_string(),
                style: Style::default().with_fg(Color::Rgb(r, g, b)),
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
