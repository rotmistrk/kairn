//! Chrome drawing for Desktop — horizontal rule lines and connectors.
//!
//! TabPanel's TabBar renders with transparent fill at row 0,
//! so the ─ line drawn here shows through in non-tab areas.

use txv_core::prelude::*;

use super::{Desktop, SlotId};

fn chrome_style() -> Style {
    txv_core::palette::palette().chrome.bar.to_style()
}

impl Desktop {
    /// Draw chrome on top of TiledWorkspace's buffer.
    pub(super) fn draw_chrome(&mut self) {
        let rects = self.layout_rects();
        let is_tall = self.is_tall();
        let is_zoomed = self.workspace.zoomed.is_some();
        let origin = self.workspace.bounds();
        let buf = self.workspace.buffer_mut();
        let w = buf.width();
        let h = buf.height();
        if w == 0 || h == 0 {
            return;
        }
        buf.fill(' ', Style::default());

        if is_zoomed {
            let cs = chrome_style();
            buf.hline(0, 0, w, '─', cs);
            return;
        }

        let cs = chrome_style();
        // Convert absolute rects to buffer-relative
        let rel = |r: Rect| -> Rect { Rect::new(r.x.saturating_sub(origin.x), r.y.saturating_sub(origin.y), r.w, r.h) };
        let left_r = rel(rects[SlotId::Left as usize]);
        let center_r = rel(rects[SlotId::Center as usize]);
        let right_r = rel(rects[SlotId::Right as usize]);
        let bottom_r = rel(rects[SlotId::Bottom as usize]);

        // Top row: full-width horizontal rule
        buf.hline(0, 0, w, '─', cs);

        // ┬ connectors where vertical dividers meet top line
        if left_r.w > 0 && center_r.w > 0 {
            let x = left_r.x + left_r.w;
            buf.put(x, 0, '┬', cs);
        }
        if right_r.w > 0 && center_r.w > 0 && !is_tall {
            let x = right_r.x.saturating_sub(1);
            buf.put(x, 0, '┬', cs);
        }

        // Vertical dividers (below chrome row)
        let tall_right = is_tall && right_r.h > 0;
        let div_y = if bottom_r.h > 0 || tall_right {
            if tall_right {
                right_r.y
            } else {
                bottom_r.y
            }
        } else {
            h
        };
        let vline_len = div_y.saturating_sub(1);
        if left_r.w > 0 && center_r.w > 0 {
            let x = left_r.x + left_r.w;
            buf.vline(x, 1, vline_len, '│', cs);
        }
        if center_r.w > 0 && right_r.w > 0 && !is_tall {
            let x = right_r.x.saturating_sub(1);
            buf.vline(x, 1, vline_len, '│', cs);
        }

        // Bottom chrome (horizontal divider — overlaps bottom panel's tab bar row)
        if (bottom_r.h > 0 || tall_right) && div_y > 0 && div_y < h {
            buf.hline(0, div_y, w, '─', cs);
            if left_r.w > 0 && center_r.w > 0 {
                let x = left_r.x + left_r.w;
                buf.put(x, div_y, '┴', cs);
            }
            if right_r.w > 0 && center_r.w > 0 && !is_tall {
                let x = right_r.x.saturating_sub(1);
                buf.put(x, div_y, '┴', cs);
            }
        }
    }
}
