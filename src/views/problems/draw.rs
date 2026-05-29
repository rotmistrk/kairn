//! Draw logic for ProblemsView.

use std::path::PathBuf;

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};

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
        view.state.mark_redrawn();
        return;
    }

    adjust_scroll(view, h);
    draw_entries(view, w, h);
    view.state.mark_redrawn();
}

fn adjust_scroll(view: &mut ProblemsView, h: usize) {
    if view.cursor < view.scroll {
        view.scroll = view.cursor;
    }
    if view.cursor >= view.scroll + h {
        view.scroll = view.cursor - h + 1;
    }
}

fn draw_entries(view: &mut ProblemsView, w: u16, h: usize) {
    let base = palette();
    let normal = base.style(StyleId::Text);
    let cursor_style = base.style(StyleId::CursorFocused);
    let root_str = view.root.to_string_lossy();

    for i in 0..h {
        let idx = view.scroll + i;
        if idx >= view.entries.len() {
            break;
        }
        let entry = &view.entries[idx];
        let y = i as u16;
        let style = if idx == view.cursor {
            cursor_style
        } else {
            normal
        };
        let buf = view.state.buffer_mut();
        buf.hline(0, y, w, ' ', style);

        let (sev_ch, sev_style) = severity_indicator(entry.severity, style);
        buf.put(1, y, sev_ch, sev_style);

        let path_str = entry.path.to_string_lossy();
        let rel = path_str
            .strip_prefix(root_str.as_ref())
            .and_then(|s| s.strip_prefix('/'))
            .unwrap_or(&path_str);
        let line_info = format!(" {}:{}  {}", rel, entry.line + 1, entry.message);
        let max = (w as usize).saturating_sub(3);
        let truncated = if line_info.len() > max {
            &line_info[..max]
        } else {
            &line_info
        };
        buf.print(2, y, truncated, style);
    }
}

fn severity_indicator(severity: Severity, base: Style) -> (char, Style) {
    let pal = palette();
    let fg = match severity {
        Severity::Error => pal.style(StyleId::StateError).fg,
        Severity::Warning => pal.style(StyleId::StateWarning).fg,
        Severity::Info => pal.style(StyleId::StateInfo).fg,
        Severity::Hint => pal.style(StyleId::StateHint).fg,
    };
    let ch = if matches!(severity, Severity::Hint) {
        '○'
    } else {
        '●'
    };
    (ch, Style { fg, ..base })
}
