//! TabGroup dropdown menu — tab selection overlay.

use txv_core::prelude::*;

use super::tab_group::TabGroup;

impl TabGroup {
    /// Open the dropdown menu.
    pub fn open_dropdown(&mut self) {
        self.dropdown_cursor = Some(self.group.focused);
        self.group.view.dirty = true;
    }

    /// Whether the dropdown is currently open.
    pub fn dropdown_open(&self) -> bool {
        self.dropdown_cursor.is_some()
    }

    /// Handle a key event while dropdown is open. Returns Consumed if handled.
    pub fn handle_dropdown_key(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        let Some(cursor) = self.dropdown_cursor else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Esc => {
                self.dropdown_cursor = None;
                self.group.view.dirty = true;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = (c as u8 - b'0') as usize;
                if idx < self.group.children.len() {
                    self.set_active(idx);
                }
                self.dropdown_cursor = None;
                self.group.view.dirty = true;
            }
            KeyCode::Enter => {
                self.set_active(cursor);
                self.dropdown_cursor = None;
                self.group.view.dirty = true;
            }
            KeyCode::Down => {
                let count = self.group.children.len();
                if count > 0 {
                    self.dropdown_cursor = Some((cursor + 1) % count);
                    self.group.view.dirty = true;
                }
            }
            KeyCode::Up => {
                let count = self.group.children.len();
                if count > 0 {
                    let prev = if cursor == 0 { count - 1 } else { cursor - 1 };
                    self.dropdown_cursor = Some(prev);
                    self.group.view.dirty = true;
                }
            }
            _ => {}
        }
        HandleResult::Consumed
    }

    /// Draw the dropdown overlay on the surface.
    pub fn draw_dropdown(&self, surface: &mut Surface) {
        let Some(cursor) = self.dropdown_cursor else {
            return;
        };
        let b = self.group.view.bounds;
        if b.w == 0 || self.titles.is_empty() {
            return;
        }
        let border = Style { fg: Color::Ansi(6), bg: Color::Ansi(0), ..Style::default() };
        let normal = Style { fg: Color::Ansi(15), bg: Color::Ansi(0), ..Style::default() };
        let cursor_style = Style {
            fg: Color::Ansi(14),
            bg: Color::Ansi(0),
            attrs: Attrs { bold: true, ..Attrs::default() },
        };

        let count = self.titles.len().min(10);
        let max_w = self.titles.iter().enumerate()
            .map(|(i, t)| format!(" {i}:{t}").len())
            .max()
            .unwrap_or(6);
        let w = ((max_w + 2) as u16).min(b.w);
        let x = b.x;
        let start_y = b.y + 1; // Below chrome
        let avail_h = (b.y + b.h).saturating_sub(start_y + 1) as usize;
        let visible = count.min(avail_h);
        let scroll = if cursor >= visible { cursor - visible + 1 } else { 0 };

        for vi in 0..visible {
            let i = scroll + vi;
            let row_y = start_y + vi as u16;
            let title = self.titles.get(i).map(|s| s.as_str()).unwrap_or("");
            let entry = format!(" {i}:{title}");
            let padded = format!("{:<width$}", entry, width = (w - 2) as usize);
            let st = if i == cursor { cursor_style } else { normal };
            surface.put(x, row_y, '│', border);
            surface.print(x + 1, row_y, &padded, st);
            if x + w > 1 {
                surface.put(x + w - 1, row_y, '│', border);
            }
        }

        // Bottom border
        let bot_y = start_y + visible as u16;
        if bot_y < b.y + b.h {
            surface.put(x, bot_y, '╰', border);
            for bx in (x + 1)..(x + w - 1) {
                surface.put(bx, bot_y, '─', border);
            }
            if x + w > 1 {
                surface.put(x + w - 1, bot_y, '╯', border);
            }
        }
    }
}
