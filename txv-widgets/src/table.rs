//! Table — columnar data display with row selection.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct Column {
    pub title: String,
    pub width: u16,
}

pub struct Table {
    state: ViewState,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
    pub cursor: usize,
    pub scroll: ScrollView,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            state: ViewState::default(),
            columns,
            rows: Vec::new(),
            cursor: 0,
            scroll: ScrollView::new(),
        }
    }

    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.scroll.set_total(self.rows.len());
        self.cursor = 0;
        self.state.dirty = true;
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds.h.saturating_sub(1) as usize; // -1 for header
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.rows.len());
        self.scroll.ensure_visible(self.cursor);
    }
}

impl View for Table {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let header_style = Style {
            attrs: Attrs {
                bold: true,
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let normal = Style::default();
        let selected = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };

        // Header row
        surface.hline(b.x, b.y, b.w, ' ', header_style);
        let mut x = b.x;
        for col in &self.columns {
            if x >= b.x + b.w {
                break;
            }
            let w = col.width.min(b.x + b.w - x);
            let title: String = col.title.chars().take(w as usize).collect();
            surface.print(x, b.y, &title, header_style);
            x += col.width;
        }

        // Data rows
        let data_h = b.h.saturating_sub(1) as usize;
        for row in 0..data_h {
            let idx = self.scroll.offset + row;
            let y = b.y + 1 + row as u16;
            if idx >= self.rows.len() {
                surface.hline(b.x, y, b.w, ' ', normal);
                continue;
            }
            let style = if idx == self.cursor {
                selected
            } else {
                normal
            };
            surface.hline(b.x, y, b.w, ' ', style);
            let mut cx = b.x;
            for (ci, col) in self.columns.iter().enumerate() {
                if cx >= b.x + b.w {
                    break;
                }
                let text = self.rows[idx].get(ci).map(|s| s.as_str()).unwrap_or("");
                let w = col.width.min(b.x + b.w - cx) as usize;
                let visible: String = text.chars().take(w).collect();
                surface.print(cx, y, &visible, style);
                cx += col.width;
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.sync_scroll();
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                let max = self.rows.len().saturating_sub(1);
                if self.cursor < max {
                    self.cursor += 1;
                    self.sync_scroll();
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                queue.put_command(CM_OK, Some(Box::new(self.cursor)));
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                let page = self.state.bounds.h.saturating_sub(2) as usize;
                let max = self.rows.len().saturating_sub(1);
                self.cursor = (self.cursor + page).min(max);
                self.sync_scroll();
                self.state.dirty = true;
                HandleResult::Consumed
            }
            KeyCode::PageUp => {
                let page = self.state.bounds.h.saturating_sub(2) as usize;
                self.cursor = self.cursor.saturating_sub(page);
                self.sync_scroll();
                self.state.dirty = true;
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
