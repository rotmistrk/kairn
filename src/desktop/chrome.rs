//! Chrome drawing for SlottedDesktop (tab bar, dividers).

use txv_core::prelude::*;
use super::{SlotId, SlottedDesktop, TOP_SLOTS};

fn chrome_style() -> Style { Style { fg: Color::Ansi(7), bg: Color::Ansi(0), attrs: Attrs::default() } }
fn focused_tab() -> Style {
    Style { fg: Color::Ansi(14), bg: Color::Ansi(4), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn active_tab() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(8), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn inactive_tab() -> Style { Style { fg: Color::Ansi(8), bg: Color::Ansi(0), attrs: Attrs::default() } }

impl SlottedDesktop {
    pub(super) fn draw_chrome(&self, surface: &mut Surface, bounds: Rect) {
        if bounds.w == 0 || bounds.h == 0 { return; }
        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);

        surface.hline(bounds.x, bounds.y, bounds.w, '─', chrome_style());
        self.draw_top_tabs(surface, &rects, bounds, tall);
        self.draw_dividers(surface, &rects, bounds);
        self.draw_bottom_chrome(surface, &rects, bounds, tall);
    }

    fn draw_top_tabs(&self, surface: &mut Surface, rects: &[Rect; 4], bounds: Rect, tall: bool) {
        for &sid in &TOP_SLOTS {
            if tall && sid == SlotId::Right { continue; }
            let slot = &self.slots[sid as usize];
            let r = rects[sid as usize];
            if r.w == 0 || slot.tabs.is_empty() { continue; }
            self.draw_slot_tabs(surface, sid, r.x, bounds.y, r.x + r.w);
        }
    }

    fn draw_slot_tabs(&self, surface: &mut Surface, sid: SlotId, start_x: u16, y: u16, max_x: u16) {
        let slot = &self.slots[sid as usize];
        let mut tx = start_x;
        for (i, (title, _)) in slot.tabs.iter().enumerate() {
            let label = format!("({})", title);
            if tx + label.len() as u16 > max_x { break; }
            let style = if i == slot.active {
                if sid == self.focused { focused_tab() } else { active_tab() }
            } else { inactive_tab() };
            surface.print(tx, y, &label, style);
            tx += label.len() as u16;
        }
    }

    fn draw_dividers(&self, surface: &mut Surface, rects: &[Rect; 4], bounds: Rect) {
        let left_r = rects[SlotId::Left as usize];
        let right_r = rects[SlotId::Right as usize];
        let center_r = rects[SlotId::Center as usize];
        let cs = chrome_style();

        if left_r.w > 0 && center_r.w > 0 {
            let div_x = left_r.x + left_r.w;
            surface.put(div_x, bounds.y, '┬', cs);
            surface.vline(div_x, bounds.y + 1, left_r.h, '│', cs);
        }
        if right_r.w > 0 && center_r.w > 0 {
            let div_x = right_r.x.saturating_sub(1);
            surface.put(div_x, bounds.y, '┬', cs);
            surface.vline(div_x, bounds.y + 1, right_r.h, '│', cs);
        }
    }

    fn draw_bottom_chrome(
        &self,
        surface: &mut Surface,
        rects: &[Rect; 4],
        bounds: Rect,
        tall: bool,
    ) {
        let bottom_r = rects[SlotId::Bottom as usize];
        if bottom_r.h == 0 { return; }

        let div_y = bottom_r.y.saturating_sub(1);
        let cs = chrome_style();
        surface.hline(bounds.x, div_y, bounds.w, '─', cs);

        let left_r = rects[SlotId::Left as usize];
        let right_r = rects[SlotId::Right as usize];
        let center_r = rects[SlotId::Center as usize];

        if left_r.w > 0 && center_r.w > 0 {
            surface.put(left_r.x + left_r.w, div_y, '┴', cs);
        }
        if right_r.w > 0 && center_r.w > 0 {
            surface.put(right_r.x.saturating_sub(1), div_y, '┴', cs);
        }

        if tall {
            self.draw_slot_tabs(
                surface, SlotId::Right, bounds.x, div_y, bounds.x + bounds.w,
            );
        }
    }
}
