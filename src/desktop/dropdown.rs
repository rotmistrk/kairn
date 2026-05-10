//! Dropdown tab picker rendering for SlottedDesktop.

use txv_core::prelude::*;

use super::{SlotId, SlottedDesktop};

impl SlottedDesktop {
    pub(super) fn draw_dropdown(&self, surface: &mut Surface, bounds: Rect) {
        let Some(slot_id) = self.dropdown else {
            return;
        };
        let slot = &self.slots[slot_id as usize];
        if slot.tabs.is_empty() {
            return;
        }

        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);
        let slot_r = if tall && slot_id == SlotId::Right {
            rects[SlotId::Bottom as usize]
        } else {
            rects[slot_id as usize]
        };

        let border = Style {
            fg: Color::Ansi(6),
            bg: Color::Ansi(0),
            attrs: Attrs::default(),
        };
        let normal = Style {
            fg: Color::Ansi(15),
            bg: Color::Ansi(0),
            attrs: Attrs::default(),
        };
        let cursor_style = Style {
            fg: Color::Ansi(14),
            bg: Color::Ansi(0),
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
        };

        // Compute dropdown width and position
        let max_name_w = slot
            .tabs
            .iter()
            .enumerate()
            .map(|(i, _)| display_width(&self.display_name(slot_id, i), 1) as usize + 4)
            .max()
            .unwrap_or(10);
        let w = (max_name_w as u16 + 2).min(slot_r.w);
        let x = slot_r.x;
        let start_y = slot_r.y + 1; // directly below slot's title bar
        let count = slot.tabs.len().min(10);
        let avail_h = (bounds.y + bounds.h).saturating_sub(start_y + 1) as usize; // -1 for bottom border
        let visible = count.min(avail_h);
        // Scroll offset: keep cursor visible
        let scroll = if self.dropdown_cursor >= visible {
            self.dropdown_cursor - visible + 1
        } else {
            0
        };

        // Draw entries (no top border — connects to title)
        for vi in 0..visible {
            let i = scroll + vi;
            let row_y = start_y + vi as u16;
            let display = self.display_name(slot_id, i);
            let entry = format!(" {}:{}", i, display);
            let padded = format!("{:<width$}", entry, width = (w - 2) as usize);
            let st = if i == self.dropdown_cursor {
                cursor_style
            } else {
                normal
            };
            // Left border
            surface.put(x, row_y, '│', border);
            // Content
            surface.print(x + 1, row_y, &padded, st);
            // Right border
            surface.put(x + w - 1, row_y, '│', border);
        }

        // Bottom border
        let bot_y = start_y + visible as u16;
        if bot_y < bounds.y + bounds.h {
            surface.put(x, bot_y, '╰', border);
            for bx in (x + 1)..(x + w - 1) {
                surface.put(bx, bot_y, '─', border);
            }
            surface.put(x + w - 1, bot_y, '╯', border);
        }
    }
}
