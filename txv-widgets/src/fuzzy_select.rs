//! FuzzySelect — input line + filtered list for fuzzy file search.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct FuzzySelect {
    state: ViewState,
    pub query: String,
    pub cursor_pos: usize,
    pub items: Vec<String>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub scroll: ScrollView,
}

impl FuzzySelect {
    pub fn new(items: Vec<String>) -> Self {
        let filtered: Vec<usize> = (0..items.len()).collect();
        let mut s = Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            query: String::new(),
            cursor_pos: 0,
            items,
            filtered,
            selected: 0,
            scroll: ScrollView::new(),
        };
        s.scroll.set_total(s.filtered.len());
        s
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.refilter();
    }

    fn refilter(&mut self) {
        self.filtered.clear();
        let q = self.query.to_lowercase();
        for (i, item) in self.items.iter().enumerate() {
            if q.is_empty() || fuzzy_match(&item.to_lowercase(), &q) {
                self.filtered.push(i);
            }
        }
        self.selected = 0;
        self.scroll.set_total(self.filtered.len());
        self.scroll.scroll_to(0);
        self.state.mark_dirty();
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h.saturating_sub(1) as usize; // -1 for input line
        self.scroll.set_viewport(h);
        self.scroll.ensure_visible(self.selected);
    }
}

/// Simple subsequence fuzzy match.
fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut chars = needle.chars();
    let mut current = chars.next();
    for h in haystack.chars() {
        if let Some(c) = current {
            if h == c {
                current = chars.next();
            }
        } else {
            break;
        }
    }
    current.is_none()
}

impl View for FuzzySelect {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let normal = Style::default();
        let selected_style = Style {
            bg: Color::Ansi(4),
            attrs: Attrs {
                underline: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let input_style = Style {
            attrs: Attrs {
                underline: true,
                ..Attrs::default()
            },
            ..Style::default()
        };

        // Input line at top
        surface.hline(b.x, b.y, b.w, ' ', input_style);
        let prompt = "> ";
        surface.print(b.x, b.y, prompt, input_style);
        let avail = b.w.saturating_sub(2) as usize;
        let visible: String = self.query.chars().take(avail).collect();
        surface.print(b.x + 2, b.y, &visible, input_style);

        // Filtered list
        let list_h = b.h.saturating_sub(1) as usize;
        for row in 0..list_h {
            let idx = self.scroll.offset + row;
            let y = b.y + 1 + row as u16;
            if idx >= self.filtered.len() {
                surface.hline(b.x, y, b.w, ' ', normal);
                continue;
            }
            let style = if idx == self.selected {
                selected_style
            } else {
                normal
            };
            surface.hline(b.x, y, b.w, ' ', style);
            let item_idx = self.filtered[idx];
            let text: String = self.items[item_idx].chars().take(b.w as usize).collect();
            surface.print(b.x + 1, y, &text, style);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match &key.code {
            KeyCode::Char(ch) => {
                self.query.insert(self.cursor_pos, *ch);
                self.cursor_pos += 1;
                self.refilter();
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.query.remove(self.cursor_pos);
                    self.refilter();
                }
                HandleResult::Consumed
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(&item_idx) = self.filtered.get(self.selected) {
                    queue.put_command(CM_OK, Some(Box::new(item_idx)));
                }
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                queue.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Consumed,
        }
    }
}
