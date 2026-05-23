//! Chrome drawing for Desktop — horizontal rule lines and connectors.
//!
//! TabPanel's TabBar renders with transparent fill at row 0,
//! so the ─ line drawn here shows through in non-tab areas.

use txv_core::prelude::*;

use super::{Desktop, SlotId, PANEL_COUNT};

fn chrome_style() -> Style {
    txv_core::palette::palette().chrome.bar.to_style()
}

/// Buffer-relative panel rects for chrome drawing.
struct ChromeRects {
    left: Rect,
    center: Rect,
    right: Rect,
    bottom: Rect,
}

impl Desktop {
    /// Draw chrome on top of TiledWorkspace's buffer.
    pub(super) fn draw_chrome(&mut self) {
        let rects = self.panel_rects();
        let is_tall = !self.workspace.is_wide();
        let is_zoomed = self.workspace.is_zoomed();
        let origin = self.workspace.bounds();
        let buf = self.workspace.buffer_mut();
        let w = buf.width();
        let h = buf.height();
        if w == 0 || h == 0 {
            return;
        }
        buf.fill(' ', Style::default());
        if is_zoomed {
            buf.hline(0, 0, w, '─', chrome_style());
            return;
        }
        let cr = Self::to_chrome_rects(&rects, origin);
        Self::draw_dividers(buf, &cr, is_tall, w, h);
    }

    fn panel_rects(&self) -> [Rect; PANEL_COUNT] {
        let mut rects = [Rect::default(); PANEL_COUNT];
        for (i, rect) in rects.iter_mut().enumerate() {
            if let Some(child) = self.workspace.child(i) {
                *rect = child.bounds();
            }
        }
        rects
    }

    fn to_chrome_rects(rects: &[Rect; PANEL_COUNT], origin: Rect) -> ChromeRects {
        let rel = |r: Rect| Rect::new(r.x.saturating_sub(origin.x), r.y.saturating_sub(origin.y), r.w, r.h);
        ChromeRects {
            left: rel(rects[SlotId::Left as usize]),
            center: rel(rects[SlotId::Center as usize]),
            right: rel(rects[SlotId::Right as usize]),
            bottom: rel(rects[SlotId::Bottom as usize]),
        }
    }

    fn draw_dividers(buf: &mut Buffer, cr: &ChromeRects, is_tall: bool, w: u16, h: u16) {
        let cs = chrome_style();
        buf.hline(0, 0, w, '─', cs);

        if cr.left.w > 0 && cr.center.w > 0 {
            buf.put(cr.left.x + cr.left.w, 0, '┬', cs);
        }
        if cr.right.w > 0 && cr.center.w > 0 && !is_tall {
            buf.put(cr.right.x.saturating_sub(1), 0, '┬', cs);
        }

        let tall_right = is_tall && cr.right.h > 0;
        let div_y = match (cr.bottom.h > 0 || tall_right, tall_right) {
            (true, true) => cr.right.y,
            (true, false) => cr.bottom.y,
            _ => h,
        };
        let vline_len = div_y.saturating_sub(1);
        if cr.left.w > 0 && cr.center.w > 0 {
            buf.vline(cr.left.x + cr.left.w, 1, vline_len, '│', cs);
        }
        if cr.center.w > 0 && cr.right.w > 0 && !is_tall {
            buf.vline(cr.right.x.saturating_sub(1), 1, vline_len, '│', cs);
        }

        if (cr.bottom.h > 0 || tall_right) && div_y > 0 && div_y < h {
            buf.hline(0, div_y, w, '─', cs);
            if cr.left.w > 0 && cr.center.w > 0 {
                buf.put(cr.left.x + cr.left.w, div_y, '┴', cs);
            }
            if cr.right.w > 0 && cr.center.w > 0 && !is_tall {
                buf.put(cr.right.x.saturating_sub(1), div_y, '┴', cs);
            }
        }
    }
}
