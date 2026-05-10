//! Chrome drawing for SlottedDesktop (tab bar, dividers).

use txv_core::prelude::*;
use super::{SlotId, SlottedDesktop, TOP_SLOTS};
use crate::glyphs::glyphs;

fn chrome_style() -> Style { Style { fg: Color::Ansi(7), bg: Color::Ansi(0), attrs: Attrs::default() } }

/// Tab title styles (focused slot)
fn focused_title() -> Style {
    Style { fg: Color::Ansi(14), bg: Color::Ansi(4), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn focused_arrow() -> Style {
    Style { fg: Color::Ansi(10), bg: Color::Ansi(4), attrs: Attrs::default() }
}
fn focused_count() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(6), attrs: Attrs { bold: true, ..Attrs::default() } }
}

/// Tab title styles (unfocused slot)
fn active_title() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(8), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn active_arrow() -> Style {
    Style { fg: Color::Ansi(7), bg: Color::Ansi(8), attrs: Attrs::default() }
}
fn active_count() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(8), attrs: Attrs::default() }
}

/// Find shortest path suffix that distinguishes `path` from `others`.
fn disambiguate_path(path: &str, others: &[&str]) -> String {
    let parts: Vec<&str> = path.rsplit('/').collect();
    for depth in 2..=parts.len() {
        let suffix: String = parts[..depth].iter().rev()
            .copied().collect::<Vec<_>>().join("/");
        let unique = others.iter().all(|other| {
            let oparts: Vec<&str> = other.rsplit('/').collect();
            let osuffix: String = oparts[..depth.min(oparts.len())].iter().rev()
                .copied().collect::<Vec<_>>().join("/");
            osuffix != suffix
        });
        if unique { return suffix; }
    }
    path.to_string()
}

impl SlottedDesktop {
    /// Compute display name for a tab, disambiguating duplicates.
    pub fn display_name(&self, slot: SlotId, idx: usize) -> String {
        let s = &self.slots[slot as usize];
        let title = match s.tabs.get(idx) {
            Some((t, _)) => t.as_str(),
            None => return String::new(),
        };
        let basename = title.rsplit('/').next().unwrap_or(title);
        let dups: Vec<&str> = s.tabs.iter().enumerate()
            .filter(|(i, (t, _))| {
                *i != idx && t.rsplit('/').next().unwrap_or(t) == basename
            })
            .map(|(_, (t, _))| t.as_str())
            .collect();
        if dups.is_empty() { return basename.to_string(); }
        disambiguate_path(title, &dups)
    }
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
        if slot.tabs.is_empty() { return; }
        let display = self.display_name(sid, slot.active);
        let focused = sid == self.focused;
        let count = slot.tabs.len();
        let g = glyphs();

        let (ts, as_, cs) = if focused {
            (focused_title(), focused_arrow(), focused_count())
        } else {
            (active_title(), active_arrow(), active_count())
        };

        let mut x = start_x;

        // Left cap (fg=title_bg on chrome_bg)
        let cap_style = Style { fg: ts.bg, bg: chrome_style().bg, attrs: Attrs::default() };
        surface.print(x, y, g.tab_left, cap_style);
        x += g.tab_left.chars().count() as u16;

        // Title text
        let title_str = format!(" {} ", display);
        if x + display_width(&title_str, 1) > max_x { return; }
        surface.print(x, y, &title_str, ts);
        x += display_width(&title_str, 1);

        if count > 1 && self.dropdown != Some(sid) {
            // Arrow
            surface.put(x, y, g.dropdown_arrow.chars().next().unwrap_or('v'), as_);
            x += 1;

            // Right cap of title / left cap of badge
            let bridge = Style { fg: ts.bg, bg: cs.bg, attrs: Attrs::default() };
            surface.print(x, y, g.tab_right, bridge);
            x += g.tab_right.chars().count() as u16;

            // Badge count
            let num = format!("{}", count);
            if x + num.len() as u16 <= max_x {
                surface.print(x, y, &num, cs);
                x += num.len() as u16;
            }

            // Right cap of badge
            let end_cap = Style { fg: cs.bg, bg: chrome_style().bg, attrs: Attrs::default() };
            surface.print(x, y, g.tab_right, end_cap);
        } else {
            // Right cap (no badge)
            let end_cap = Style { fg: ts.bg, bg: chrome_style().bg, attrs: Attrs::default() };
            surface.print(x, y, g.tab_right, end_cap);
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

    pub(super) fn draw_dropdown(&self, surface: &mut Surface, bounds: Rect) {
        let Some(slot_id) = self.dropdown else { return; };
        let slot = &self.slots[slot_id as usize];
        if slot.tabs.is_empty() { return; }

        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);
        let slot_r = if tall && slot_id == SlotId::Right {
            rects[SlotId::Bottom as usize]
        } else {
            rects[slot_id as usize]
        };

        let border = Style { fg: Color::Ansi(6), bg: Color::Ansi(0), attrs: Attrs::default() };
        let normal = Style { fg: Color::Ansi(15), bg: Color::Ansi(0), attrs: Attrs::default() };
        let cursor_style = Style {
            fg: Color::Ansi(14), bg: Color::Ansi(0),
            attrs: Attrs { bold: true, ..Attrs::default() },
        };

        // Compute dropdown width and position
        let max_name_w = slot.tabs.iter().enumerate()
            .map(|(i, _)| display_width(&self.display_name(slot_id, i), 1) as usize + 4)
            .max().unwrap_or(10);
        let w = (max_name_w as u16 + 2).min(slot_r.w);
        let x = slot_r.x;
        let start_y = bounds.y + 1; // directly below title bar (open top)
        let count = slot.tabs.len().min(10);

        // Draw entries (no top border — connects to title)
        for i in 0..count {
            let row_y = start_y + i as u16;
            if row_y >= bounds.y + bounds.h { break; }
            let display = self.display_name(slot_id, i);
            let entry = format!(" {}:{}", i, display);
            let padded = format!("{:<width$}", entry, width = (w - 2) as usize);
            let st = if i == self.dropdown_cursor { cursor_style } else { normal };
            // Left border
            surface.put(x, row_y, '│', border);
            // Content
            surface.print(x + 1, row_y, &padded, st);
            // Right border
            surface.put(x + w - 1, row_y, '│', border);
        }

        // Bottom border
        let bot_y = start_y + count as u16;
        if bot_y < bounds.y + bounds.h {
            surface.put(x, bot_y, '╰', border);
            for bx in (x + 1)..(x + w - 1) {
                surface.put(bx, bot_y, '─', border);
            }
            surface.put(x + w - 1, bot_y, '╯', border);
        }
    }
}
