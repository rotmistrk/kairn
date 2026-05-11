//! View trait implementation for LayoutGroup.

use txv_core::event::KeyCode;
use txv_core::prelude::*;

use super::{LayoutGroup, SlotId};

impl View for LayoutGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        self.group.view.dirty = true;
        self.apply_layout(r);
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.group.view.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        if let Some(z) = self.zoomed {
            // Zoomed panel is on top — only draw it
            self.group.children[z].draw(surface);
            return;
        }
        for child in &self.group.children {
            let pb = child.bounds();
            if pb.w > 0 && pb.h > 0 {
                child.draw(surface);
            }
        }
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
        let dim = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let rects = self.compute_rects(b);
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        if left_r.w > 0 && center_r.w > 0 {
            let x = left_r.x + left_r.w;
            for y in left_r.y..left_r.y + left_r.h {
                surface.put(x, y, '│', dim);
            }
        }
        if center_r.w > 0 && right_r.w > 0 {
            let x = center_r.x + center_r.w;
            for y in center_r.y..center_r.y + center_r.h {
                surface.put(x, y, '│', dim);
            }
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
        let style = Style {
            fg: Color::Ansi(15),
            bg: Color::Ansi(0),
            ..Style::default()
        };
        let hl = Style {
            fg: Color::Ansi(0),
            bg: Color::Ansi(14),
            ..Style::default()
        };
        let x = pb.x;
        let start_y = pb.y + 1;
        let w = pb.w.min(30);
        for i in 0..panel.tab_count() {
            let y = start_y + i as u16;
            if y >= pb.y + pb.h {
                break;
            }
            let s = if i == self.dropdown_cursor {
                hl
            } else {
                style
            };
            surface.hline(x, y, w, ' ', s);
            let title = panel.tab_title(i).unwrap_or("");
            let label = format!("{i}:{title}");
            surface.print(x, y, &label, s);
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
