//! View trait implementation for LayoutGroup.

use txv_core::event::KeyCode;
use txv_core::prelude::*;

use super::{LayoutGroup, SlotId};

impl View for LayoutGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        self.group.view.dirty = true;
        // Recompute proportional sizes on terminal resize
        // Wide: 1:2:2 by width. Tall: 1:2 width, 2:1 height.
        if r.w > 0 {
            let usable_w = r.w.saturating_sub(2); // max 2 dividers
            self.left_width = usable_w / 5;
            self.right_width = usable_w * 2 / 5;
        }
        if r.h > 0 {
            self.right_height = r.h * 2 / 3;
            self.bottom_height = r.h / 3;
        }
        self.apply_layout(r);
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.group.view.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        if let Some(z) = self.zoomed {
            // Zoomed panel is on top — draw it with chrome and dropdown
            self.group.children[z].draw(surface);
            self.draw_zoomed_chrome(surface, z);
            self.draw_dropdown(surface);
            return;
        }
        for child in &self.group.children {
            let pb = child.bounds();
            if pb.w > 0 && pb.h > 0 {
                child.draw(surface);
            }
        }
        // Chrome overwrites TabGroup's plain chrome with Powerline visuals
        self.draw_chrome(surface);
        self.draw_dividers(surface, b);
        self.draw_dropdown(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Dropdown intercepts all keys when open
        if self.dropdown.is_some() {
            if let Event::Key(key) = event {
                return self.handle_dropdown_key(key);
            }
        }
        // Alt-digit for tab selection
        if let Event::Key(key) = event {
            if key.modifiers.alt && !key.modifiers.ctrl {
                if let KeyCode::Char(ch) = key.code {
                    if let Some(n) = ch.to_digit(10) {
                        let slot = self.focused_slot();
                        if (n as usize) < self.panel(slot).tab_count() {
                            self.panel_mut(slot).set_active(n as usize);
                        }
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        // Commands handled by LayoutGroup itself
        if let Event::Command { id, .. } = event {
            let r = self.handle_command(*id, queue);
            if r == HandleResult::Consumed {
                return r;
            }
        }
        // Delegate to focused child via GroupState 3-phase dispatch
        self.group.dispatch(event, queue)
    }
}

impl LayoutGroup {
    fn draw_dividers(&self, surface: &mut Surface, b: Rect) {
        let cs = Style {
            fg: Color::Ansi(7),
            bg: Color::Ansi(0),
            ..Style::default()
        };
        let rects = self.compute_rects(b);
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        // Vertical dividers (below chrome row)
        if left_r.w > 0 && center_r.w > 0 {
            let x = left_r.x + left_r.w;
            surface.vline(x, b.y + 1, left_r.h.saturating_sub(1), '│', cs);
        }
        if center_r.w > 0 && right_r.w > 0 {
            let x = right_r.x.saturating_sub(1);
            surface.vline(x, b.y + 1, right_r.h.saturating_sub(1), '│', cs);
        }
    }

    fn draw_dropdown(&self, surface: &mut Surface) {
        let Some(panel_idx) = self.dropdown else {
            return;
        };
        let panel = self.panel(Self::slot_from(panel_idx));
        let pb = self.group.children[panel_idx].bounds();
        if pb.w == 0 || panel.tab_count() == 0 {
            return;
        }
        let border = Style {
            fg: Color::Ansi(6),
            bg: Color::Ansi(0),
            ..Style::default()
        };
        let normal = Style {
            fg: Color::Ansi(15),
            bg: Color::Ansi(0),
            ..Style::default()
        };
        let cursor_style = Style {
            fg: Color::Ansi(14),
            bg: Color::Ansi(0),
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
        };

        let count = panel.tab_count().min(10);
        // Width fits longest entry: " N:title" + 2 for borders
        let max_entry_w = (0..panel.tab_count())
            .map(|i| {
                let t = panel.tab_title(i).unwrap_or("");
                format!(" {i}:{t}").len()
            })
            .max()
            .unwrap_or(6);
        let w = ((max_entry_w + 2) as u16).min(pb.w);
        let x = pb.x;
        let start_y = pb.y + 1; // Below chrome
        let avail_h = (pb.y + pb.h).saturating_sub(start_y + 1) as usize;
        let visible = count.min(avail_h);
        let scroll = if self.dropdown_cursor >= visible {
            self.dropdown_cursor - visible + 1
        } else {
            0
        };

        for vi in 0..visible {
            let i = scroll + vi;
            let row_y = start_y + vi as u16;
            let title = panel.tab_title(i).unwrap_or("");
            let entry = format!(" {i}:{title}");
            let padded = format!("{:<width$}", entry, width = (w - 2) as usize);
            let st = if i == self.dropdown_cursor {
                cursor_style
            } else {
                normal
            };
            surface.put(x, row_y, '│', border);
            surface.print(x + 1, row_y, &padded, st);
            if x + w > 1 {
                surface.put(x + w - 1, row_y, '│', border);
            }
        }

        // Bottom border
        let bot_y = start_y + visible as u16;
        let bounds = self.group.view.bounds;
        if bot_y < bounds.y + bounds.h {
            surface.put(x, bot_y, '╰', border);
            for bx in (x + 1)..(x + w - 1) {
                surface.put(bx, bot_y, '─', border);
            }
            if x + w > 1 {
                surface.put(x + w - 1, bot_y, '╯', border);
            }
        }
    }

    pub(super) fn handle_dropdown_key(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        let slot = self.focused_slot();
        match key.code {
            KeyCode::Esc => {
                self.dropdown = None;
                self.group.view.dirty = true;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = (c as u8 - b'0') as usize;
                if idx < self.panel(slot).tab_count() {
                    self.panel_mut(slot).set_active(idx);
                }
                self.dropdown = None;
                self.group.view.dirty = true;
            }
            KeyCode::Enter => {
                let cursor = self.dropdown_cursor;
                self.panel_mut(slot).set_active(cursor);
                self.dropdown = None;
                self.group.view.dirty = true;
            }
            KeyCode::Down => {
                let count = self.panel(slot).tab_count();
                if count > 0 {
                    self.dropdown_cursor = (self.dropdown_cursor + 1) % count;
                    self.group.view.dirty = true;
                }
            }
            KeyCode::Up => {
                let count = self.panel(slot).tab_count();
                if count > 0 {
                    self.dropdown_cursor = if self.dropdown_cursor == 0 {
                        count - 1
                    } else {
                        self.dropdown_cursor - 1
                    };
                    self.group.view.dirty = true;
                }
            }
            _ => {}
        }
        HandleResult::Consumed
    }
}
