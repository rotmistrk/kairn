//! Chrome drawing for LayoutGroup — Powerline tab bar, dividers, connectors.

use txv_core::prelude::*;

use super::{LayoutGroup, SlotId, PANEL_COUNT};
use crate::glyphs::glyphs;

fn chrome_style() -> Style {
    Style {
        fg: Color::Ansi(7),
        bg: Color::Ansi(0),
        attrs: Attrs::default(),
    }
}

fn focused_title() -> Style {
    Style {
        fg: Color::Ansi(14),
        bg: Color::Ansi(4),
        attrs: Attrs {
            bold: true,
            ..Attrs::default()
        },
    }
}

fn focused_arrow() -> Style {
    Style {
        fg: Color::Ansi(10),
        bg: Color::Ansi(4),
        attrs: Attrs::default(),
    }
}

fn focused_count() -> Style {
    Style {
        fg: Color::Ansi(15),
        bg: Color::Ansi(6),
        attrs: Attrs {
            bold: true,
            ..Attrs::default()
        },
    }
}

fn active_title() -> Style {
    Style {
        fg: Color::Ansi(15),
        bg: Color::Ansi(8),
        attrs: Attrs {
            bold: true,
            ..Attrs::default()
        },
    }
}

fn active_arrow() -> Style {
    Style {
        fg: Color::Ansi(7),
        bg: Color::Ansi(8),
        attrs: Attrs::default(),
    }
}

fn active_count() -> Style {
    Style {
        fg: Color::Ansi(15),
        bg: Color::Ansi(8),
        attrs: Attrs::default(),
    }
}

impl LayoutGroup {
    /// Draw the full chrome bar (top row) with Powerline glyphs.
    pub(super) fn draw_chrome(&self, surface: &mut Surface) {
        let b = self.group.view.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let rects = self.compute_rects(b);
        let tall = self.is_tall();
        let cs = chrome_style();

        // Fill top row with ─
        surface.hline(b.x, b.y, b.w, '─', cs);

        // Draw tabs for top slots
        for (i, r) in rects[..3].iter().enumerate() {
            if tall && i == SlotId::Right as usize {
                continue;
            }
            if r.w == 0 {
                continue;
            }
            let panel = self.panel(Self::slot_from(i));
            if panel.tab_count() == 0 {
                continue;
            }
            let focused = i == self.group.focused;
            self.draw_slot_tab(surface, i, r.x, b.y, r.x + r.w, focused);
        }

        // Divider connectors (┬)
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        if left_r.w > 0 && center_r.w > 0 {
            surface.put(left_r.x + left_r.w, b.y, '┬', cs);
        }
        if right_r.w > 0 && center_r.w > 0 {
            surface.put(right_r.x.saturating_sub(1), b.y, '┬', cs);
        }

        // Bottom chrome (horizontal divider above bottom panel)
        self.draw_bottom_chrome(surface, &rects, b, tall);
    }

    fn draw_slot_tab(&self, surface: &mut Surface, panel_idx: usize, start_x: u16, y: u16, max_x: u16, focused: bool) {
        let panel = self.panel(Self::slot_from(panel_idx));
        let title = panel.active_title().unwrap_or("");
        let count = panel.tab_count();
        let g = glyphs();
        let cs = chrome_style();

        let (ts, _as, _cs) = if focused {
            (focused_title(), focused_arrow(), focused_count())
        } else {
            (active_title(), active_arrow(), active_count())
        };

        let mut x = start_x;

        // Left cap
        let cap = Style {
            fg: ts.bg,
            bg: cs.bg,
            ..Style::default()
        };
        surface.print(x, y, g.tab_left, cap);
        x += g.tab_left.chars().count() as u16;

        // Title
        let label = format!(" {title} ");
        let lw = display_width(&label, 1);
        if x + lw > max_x {
            return;
        }
        surface.print(x, y, &label, ts);
        x += lw;

        if count > 1 && self.dropdown.is_none() {
            // Arrow
            surface.put(x, y, g.dropdown_arrow.chars().next().unwrap_or('v'), _as);
            x += 1;
            // Bridge (title bg → count bg)
            let bridge = Style {
                fg: ts.bg,
                bg: _cs.bg,
                ..Style::default()
            };
            surface.print(x, y, g.tab_right, bridge);
            x += g.tab_right.chars().count() as u16;
            // Badge count
            let num = format!("{count}");
            if x + num.len() as u16 <= max_x {
                surface.print(x, y, &num, _cs);
                x += num.len() as u16;
            }
            // End cap
            let end = Style {
                fg: _cs.bg,
                bg: cs.bg,
                ..Style::default()
            };
            surface.print(x, y, g.tab_right, end);
        } else {
            // Right cap (no badge)
            let end = Style {
                fg: ts.bg,
                bg: cs.bg,
                ..Style::default()
            };
            surface.print(x, y, g.tab_right, end);
        }
    }

    fn draw_bottom_chrome(&self, surface: &mut Surface, rects: &[Rect; PANEL_COUNT], b: Rect, tall: bool) {
        let bottom_r = rects[SlotId::Bottom as usize];
        if bottom_r.h == 0 && !(tall && self.panel(SlotId::Right).tab_count() > 0) {
            return;
        }
        // In tall mode, right panel is below — its bounds start at the divider row
        let div_y = if tall {
            let right_bounds = self.group.children[SlotId::Right as usize].bounds();
            if right_bounds.h == 0 {
                return;
            }
            right_bounds.y
        } else if bottom_r.h > 0 {
            bottom_r.y
        } else {
            return;
        };

        let cs = chrome_style();
        surface.hline(b.x, div_y, b.w, '─', cs);

        // ┴ connectors where vertical dividers meet horizontal
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        if left_r.w > 0 && center_r.w > 0 {
            surface.put(left_r.x + left_r.w, div_y, '┴', cs);
        }
        if right_r.w > 0 && center_r.w > 0 {
            surface.put(right_r.x.saturating_sub(1), div_y, '┴', cs);
        }

        // Draw tab for the bottom slot
        if tall {
            let focused = self.group.focused == SlotId::Right as usize;
            self.draw_slot_tab(surface, SlotId::Right as usize, b.x, div_y, b.x + b.w, focused);
        }
    }

    /// Draw chrome for a single zoomed panel (full-width Powerline tab bar).
    pub(super) fn draw_zoomed_chrome(&self, surface: &mut Surface, panel_idx: usize) {
        let b = self.group.view.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let cs = chrome_style();
        surface.hline(b.x, b.y, b.w, '─', cs);
        let panel = self.panel(Self::slot_from(panel_idx));
        if panel.tab_count() > 0 {
            self.draw_slot_tab(surface, panel_idx, b.x, b.y, b.x + b.w, true);
        }
    }
}
