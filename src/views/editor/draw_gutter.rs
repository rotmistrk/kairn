//! Gutter rendering: line numbers, git signs, diagnostic markers.

use txv_core::prelude::*;

use super::draw::DrawParams;
use super::EditorView;
use crate::app_palette::app_palette;
use crate::gutter_signs::GutterSign;
use crate::views::editor::draw_diagnostics::diag_marker_style;

fn gutter_sign_style(sign: GutterSign) -> (char, Style) {
    let app = app_palette();
    match sign {
        GutterSign::Added => ('▎', app.diff().added()),
        GutterSign::Modified => ('▎', app.git().modified()),
        GutterSign::Deleted => ('▸', app.diff().deleted()),
    }
}

impl EditorView {
    pub(super) fn draw_gutter(&mut self, line_idx: usize, y: u16, p: &DrawParams) {
        if p.gutter_w == 0 {
            return;
        }
        let sign_w: usize = if self.editor.options().gutter_signs() {
            1
        } else {
            0
        };
        if sign_w > 0 {
            if let Some(sign) = self.gutter_sign_at(line_idx) {
                let (ch, style) = gutter_sign_style(sign);
                self.state.buffer_mut().put(0, y, ch, style);
            } else {
                self.state.buffer_mut().put(0, y, ' ', p.gutter_style);
            }
        }
        let num_w = p.gutter_w as usize - sign_w;
        let num = format!("{:>width$} ", line_idx + 1, width = num_w.saturating_sub(1));
        self.state.buffer_mut().print(sign_w as u16, y, &num, p.gutter_style);
        if let Some(sev) = self.diagnostic_severity_at(line_idx) {
            let marker_style = diag_marker_style(sev);
            self.state.buffer_mut().put(p.gutter_w - 1, y, '●', marker_style);
        }
    }

    fn gutter_sign_at(&self, line_idx: usize) -> Option<GutterSign> {
        self.gutter_signs.iter().find(|(l, _)| *l == line_idx).map(|(_, s)| *s)
    }
}
