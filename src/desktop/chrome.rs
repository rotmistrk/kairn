//! Chrome drawing for SlottedDesktop (tab bar, dividers).

use txv_core::prelude::*;
use super::{SlotId, SlottedDesktop, TOP_SLOTS};

impl SlottedDesktop {
    pub(super) fn draw_chrome(&self, surface: &mut Surface, bounds: Rect) {
        if bounds.w == 0 || bounds.h == 0 { return; }
        let rects = self.layout(bounds);
        let chrome_style = Style { fg: Color::Ansi(7), bg: Color::Ansi(0), attrs: Attrs::default() };
        let focused_tab_style = Style {
            fg: Color::Ansi(14), bg: Color::Ansi(4),
            attrs: Attrs { bold: true, ..Attrs::default() },
        };
        let unfocused_active_style = Style {
            fg: Color::Ansi(15), bg: Color::Ansi(8),
            attrs: Attrs { bold: true, ..Attrs::default() },
        };
        let inactive_tab_style = Style { fg: Color::Ansi(8), bg: Color::Ansi(0), attrs: Attrs::default() };

        let y = bounds.y;
        surface.hline(bounds.x, y, bounds.w, '─', chrome_style);

        for &sid in &TOP_SLOTS {
            let slot = &self.slots[sid as usize];
            let r = rects[sid as usize];
            if r.w == 0 || slot.tabs.is_empty() { continue; }
            let mut tx = r.x;
            for (i, (title, _)) in slot.tabs.iter().enumerate() {
                let label = format!("({})", title);
                let style = if i == slot.active {
                    if sid == self.focused { focused_tab_style } else { unfocused_active_style }
                } else { inactive_tab_style };
                if tx + label.len() as u16 > r.x + r.w { break; }
                surface.print(tx, y, &label, style);
                tx += label.len() as u16;
            }
        }

        let left_r = rects[SlotId::Left as usize];
        let right_r = rects[SlotId::Right as usize];
        let center_r = rects[SlotId::Center as usize];

        if left_r.w > 0 && center_r.w > 0 {
            let div_x = left_r.x + left_r.w;
            surface.put(div_x, y, '┬', chrome_style);
            surface.vline(div_x, y + 1, left_r.h, '│', chrome_style);
        }
        if right_r.w > 0 && center_r.w > 0 {
            let div_x = right_r.x.saturating_sub(1);
            surface.put(div_x, y, '┬', chrome_style);
            surface.vline(div_x, y + 1, right_r.h, '│', chrome_style);
        }

        let bottom_r = rects[SlotId::Bottom as usize];
        if bottom_r.h > 0 {
            let div_y = bottom_r.y.saturating_sub(1);
            surface.hline(bounds.x, div_y, bounds.w, '─', chrome_style);
            if left_r.w > 0 && center_r.w > 0 {
                surface.put(left_r.x + left_r.w, div_y, '┴', chrome_style);
            }
            if right_r.w > 0 && center_r.w > 0 {
                surface.put(right_r.x.saturating_sub(1), div_y, '┴', chrome_style);
            }
        }
    }
}
