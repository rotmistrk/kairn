//! Draw logic for ProblemsView.

use std::path::PathBuf;

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::view::ViewState;

use crate::lsp::diagnostics::Severity;

use super::ProblemsView;

/// A flattened diagnostic entry for display.
pub(super) struct Entry {
    pub(super) path: PathBuf,
    pub(super) line: usize,
    pub(super) severity: Severity,
    pub(super) message: String,
}

pub(super) fn draw(view: &mut ProblemsView) {
    let buf = view.state.buffer_mut();
    let w = buf.width();
    let h = buf.height() as usize;
    if w == 0 || h == 0 {
        return;
    }
    let base = palette();
    let normal = base.style(StyleId::Text);
    buf.fill(' ', normal);

    if view.entries.is_empty() {
        buf.print(1, 0, "No problems", normal);
        return;
    }

    adjust_scroll(view, h);
    draw_entries(view, w, h);
}

fn adjust_scroll(view: &mut ProblemsView, h: usize) {
    if h == 0 {
        return;
    }
    if view.cursor < view.scroll {
        view.scroll = view.cursor;
        return;
    }
    while !cursor_visible(view, h) {
        view.scroll += 1;
        if view.scroll > view.cursor {
            view.scroll = view.cursor;
            break;
        }
    }
}

fn cursor_visible(view: &ProblemsView, h: usize) -> bool {
    let mut row = 0;
    for idx in view.scroll..view.entries.len() {
        let lines = if idx == view.cursor {
            selected_line_count(view)
        } else {
            1
        };
        if idx == view.cursor {
            return row + lines <= h;
        }
        row += lines;
        if row >= h {
            return false;
        }
    }
    false
}

fn selected_line_count(view: &ProblemsView) -> usize {
    view.entries
        .get(view.cursor)
        .map(|e| e.message.lines().count().max(1))
        .unwrap_or(1)
}

fn draw_entries(view: &mut ProblemsView, w: u16, h: usize) {
    let base = palette();
    let normal = base.style(StyleId::Text);
    let cursor_style = base.style(StyleId::CursorFocused);
    let root_str = view.root.to_string_lossy();

    let mut y: usize = 0;
    let mut idx = view.scroll;
    while y < h && idx < view.entries.len() {
        let selected = idx == view.cursor;
        let style = if selected {
            cursor_style
        } else {
            normal
        };
        let entry = &view.entries[idx];

        let path_str = entry.path.to_string_lossy();
        let rel = path_str
            .strip_prefix(root_str.as_ref())
            .and_then(|s| s.strip_prefix('/'))
            .unwrap_or(&path_str);
        let prefix = format!(" {}:{} ", rel, entry.line + 1);

        if selected {
            y += draw_expanded(&mut view.state, entry, &prefix, style, w, y, h);
        } else {
            draw_collapsed(&mut view.state, entry, &prefix, style, w, y as u16);
            y += 1;
        }
        idx += 1;
    }
}

fn draw_expanded(
    state: &mut ViewState,
    entry: &Entry,
    prefix: &str,
    style: Style,
    w: u16,
    start_y: usize,
    h: usize,
) -> usize {
    let (sev_ch, sev_style) = severity_indicator(entry.severity, style);
    let max_msg = (w as usize).saturating_sub(3 + prefix.len());
    let mut drawn = 0;
    for (li, msg_line) in entry.message.lines().enumerate() {
        if start_y + drawn >= h {
            break;
        }
        let row = (start_y + drawn) as u16;
        let buf = state.buffer_mut();
        buf.hline(0, row, w, ' ', style);
        if li == 0 {
            buf.put(1, row, sev_ch, sev_style);
            buf.print(2, row, prefix, style);
            buf.print((2 + prefix.len()) as u16, row, truncate(msg_line, max_msg), style);
        } else {
            let indent = 3 + prefix.len();
            buf.print(
                indent as u16,
                row,
                truncate(msg_line, (w as usize).saturating_sub(indent)),
                style,
            );
        }
        drawn += 1;
    }
    drawn
}

fn draw_collapsed(state: &mut ViewState, entry: &Entry, prefix: &str, style: Style, w: u16, row: u16) {
    let (sev_ch, sev_style) = severity_indicator(entry.severity, style);
    let max_msg = (w as usize).saturating_sub(3 + prefix.len());
    let msg_first = entry.message.lines().next().unwrap_or("");
    let buf = state.buffer_mut();
    buf.hline(0, row, w, ' ', style);
    buf.put(1, row, sev_ch, sev_style);
    buf.print(2, row, prefix, style);
    buf.print((2 + prefix.len()) as u16, row, truncate(msg_first, max_msg), style);
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

fn severity_indicator(severity: Severity, base: Style) -> (char, Style) {
    let pal = palette();
    let fg = match severity {
        Severity::Error => pal.style(StyleId::StateError).fg(),
        Severity::Warning => pal.style(StyleId::StateWarning).fg(),
        Severity::Info => pal.style(StyleId::StateInfo).fg(),
        Severity::Hint => pal.style(StyleId::StateHint).fg(),
    };
    let ch = if matches!(severity, Severity::Hint) {
        '○'
    } else {
        '●'
    };
    (ch, Style::new(fg, base.bg()).with_attrs(base.attrs()))
}
