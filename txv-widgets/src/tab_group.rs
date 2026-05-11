//! TabGroup — a View that manages a stack of tabbed child views.
//!
//! Only the active tab's view is drawn and receives events.
//! Chrome (tab title bar) is drawn at the top row.

use txv_core::prelude::*;

/// A tabbed container — owns multiple views, shows one at a time.
pub struct TabGroup {
    state: ViewState,
    tabs: Vec<(String, Box<dyn View>)>,
    active: usize,
    lru: Vec<u64>,
    lru_counter: u64,
}

impl TabGroup {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            tabs: Vec::new(),
            active: 0,
            lru: Vec::new(),
            lru_counter: 0,
        }
    }

    pub fn insert_tab(&mut self, title: impl Into<String>, mut view: Box<dyn View>) {
        let content_rect = self.content_rect();
        view.set_bounds(content_rect);
        self.tabs.push((title.into(), view));
        self.lru.push(0);
        self.state.dirty = true;
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.tabs.len() || index == self.active {
            return;
        }
        self.tabs[self.active].1.unselect();
        self.active = index;
        self.touch_lru();
        let r = self.content_rect();
        self.tabs[self.active].1.set_bounds(r);
        if self.state.focused {
            self.tabs[self.active].1.select();
        }
        self.state.dirty = true;
    }

    pub fn active_title(&self) -> Option<&str> {
        self.tabs.get(self.active).map(|(t, _)| t.as_str())
    }

    pub fn active_view_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.tabs.get_mut(self.active).map(|(_, v)| v)
    }

    pub fn tab_next(&mut self) {
        if self.tabs.len() > 1 {
            self.set_active((self.active + 1) % self.tabs.len());
        }
    }

    pub fn tab_prev(&mut self) {
        if self.tabs.len() > 1 {
            let prev = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
            self.set_active(prev);
        }
    }

    pub fn close_active(&mut self) -> bool {
        if self.tabs.is_empty() {
            return false;
        }
        if let CloseResult::Denied(_) = self.tabs[self.active].1.can_close() {
            return false;
        }
        self.tabs.remove(self.active);
        self.lru.remove(self.active);
        self.adjust_after_remove();
        true
    }

    pub fn close_tab_by_title(&mut self, title: &str) -> bool {
        let Some(idx) = self.tabs.iter().position(|(t, _)| t == title) else {
            return false;
        };
        if let CloseResult::Denied(_) = self.tabs[idx].1.can_close() {
            return false;
        }
        self.tabs.remove(idx);
        self.lru.remove(idx);
        self.adjust_after_remove();
        true
    }

    pub fn focus_tab_by_title(&mut self, title: &str) -> bool {
        if let Some(idx) = self.tabs.iter().position(|(t, _)| t == title) {
            self.set_active(idx);
            true
        } else {
            false
        }
    }

    fn content_rect(&self) -> Rect {
        let b = self.state.bounds;
        Rect::new(b.x, b.y + 1, b.w, b.h.saturating_sub(1))
    }

    fn touch_lru(&mut self) {
        self.lru_counter += 1;
        if let Some(v) = self.lru.get_mut(self.active) {
            *v = self.lru_counter;
        }
    }

    fn adjust_after_remove(&mut self) {
        if self.active >= self.tabs.len() && self.active > 0 {
            self.active -= 1;
        }
        if !self.lru.is_empty() {
            let mru = self
                .lru
                .iter()
                .enumerate()
                .max_by_key(|(_, &v)| v)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.active = mru;
        }
        if let Some((_, v)) = self.tabs.get_mut(self.active) {
            let b = self.state.bounds;
            let r = Rect::new(b.x, b.y + 1, b.w, b.h.saturating_sub(1));
            v.set_bounds(r);
            if self.state.focused {
                v.select();
            }
        }
        self.state.dirty = true;
    }

    fn draw_chrome(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let dim = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let bright = Style {
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        surface.hline(b.x, b.y, b.w, ' ', dim);
        let mut x = b.x;
        for (i, (title, _)) in self.tabs.iter().enumerate() {
            let style = if i == self.active {
                bright
            } else {
                dim
            };
            let label = format!(" {title} ");
            let len = label.len() as u16;
            if x + len > b.x + b.w {
                break;
            }
            surface.print(x, b.y, &label, style);
            x += len;
        }
    }
}

impl Default for TabGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TabGroup {
    fn bounds(&self) -> Rect {
        self.state.bounds
    }
    fn set_bounds(&mut self, r: Rect) {
        self.state.bounds = r;
        let content = self.content_rect();
        if let Some((_, view)) = self.tabs.get_mut(self.active) {
            view.set_bounds(content);
        }
        self.state.dirty = true;
    }
    fn options(&self) -> ViewOptions {
        self.state.options
    }
    fn title(&self) -> &str {
        self.active_title().unwrap_or("")
    }
    fn needs_redraw(&self) -> bool {
        self.state.dirty || self.tabs.get(self.active).is_some_and(|(_, v)| v.needs_redraw())
    }
    fn mark_redrawn(&mut self) {
        self.state.dirty = false;
        if let Some((_, v)) = self.tabs.get_mut(self.active) {
            v.mark_redrawn();
        }
    }
    fn select(&mut self) {
        self.state.focused = true;
        self.state.dirty = true;
        if let Some((_, v)) = self.tabs.get_mut(self.active) {
            v.select();
        }
    }
    fn unselect(&mut self) {
        self.state.focused = false;
        self.state.dirty = true;
        if let Some((_, v)) = self.tabs.get_mut(self.active) {
            v.unselect();
        }
    }
    fn draw(&self, surface: &mut Surface) {
        self.draw_chrome(surface);
        if let Some((_, view)) = self.tabs.get(self.active) {
            view.draw(surface);
        }
    }
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Some((_, view)) = self.tabs.get_mut(self.active) {
            let result = view.handle(event, queue);
            if result == HandleResult::Consumed {
                return result;
            }
        }
        HandleResult::Ignored
    }
}

#[cfg(test)]
#[path = "tab_group_tests.rs"]
mod tests;
