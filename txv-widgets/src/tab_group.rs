//! TabGroup — a View that manages a stack of tabbed child views.
//!
//! Uses GroupState for child management and event dispatch.
//! Only the active tab's view is drawn and receives events.
//! Chrome (tab title bar) is drawn at the top row.

use txv_core::prelude::*;

/// A tabbed container — owns multiple views, shows one at a time.
pub struct TabGroup {
    pub(crate) group: GroupState,
    pub(crate) titles: Vec<String>,
    pub(crate) lru: Vec<u64>,
    pub(crate) lru_counter: u64,
    /// Dropdown menu state: Some(cursor_index) when open.
    pub dropdown_cursor: Option<usize>,
}

impl TabGroup {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            titles: Vec::new(),
            lru: Vec::new(),
            lru_counter: 0,
            dropdown_cursor: None,
        }
    }

    pub fn insert_tab(&mut self, title: impl Into<String>, mut view: Box<dyn View>) {
        let content_rect = self.content_rect();
        view.set_bounds(content_rect);
        // Unselect previous active child
        if let Some(child) = self.group.children.get_mut(self.group.focused) {
            child.unselect();
        }
        self.group.children.push(view);
        self.titles.push(title.into());
        self.lru.push(0);
        self.group.focused = self.group.children.len() - 1;
        self.touch_lru();
        if self.group.view.focused {
            if let Some(child) = self.group.children.get_mut(self.group.focused) {
                child.select();
            }
        }
        self.group.view.dirty = true;
    }

    pub fn tab_count(&self) -> usize {
        self.group.children.len()
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.group.children.len() || index == self.group.focused {
            return;
        }
        self.group.children[self.group.focused].unselect();
        self.group.focused = index;
        self.touch_lru();
        let r = self.content_rect();
        self.group.children[self.group.focused].set_bounds(r);
        if self.group.view.focused {
            self.group.children[self.group.focused].select();
        }
        self.group.view.dirty = true;
    }

    pub fn active_title(&self) -> Option<&str> {
        self.titles.get(self.group.focused).map(|t| t.as_str())
    }

    /// Index of the currently active tab.
    pub fn active_index(&self) -> usize {
        self.group.focused
    }

    pub fn tab_title(&self, index: usize) -> Option<&str> {
        self.titles.get(index).map(|t| t.as_str())
    }

    pub fn active_view_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.group.children.get_mut(self.group.focused)
    }

    pub fn tab_next(&mut self) {
        if self.group.children.len() > 1 {
            self.set_active((self.group.focused + 1) % self.group.children.len());
        }
    }

    pub fn tab_prev(&mut self) {
        if self.group.children.len() > 1 {
            let prev = if self.group.focused == 0 {
                self.group.children.len() - 1
            } else {
                self.group.focused - 1
            };
            self.set_active(prev);
        }
    }

    pub fn close_active(&mut self) -> bool {
        if self.group.children.is_empty() {
            return false;
        }
        if let CloseResult::Denied(_) = self.group.children[self.group.focused].can_close() {
            return false;
        }
        self.group.children.remove(self.group.focused);
        self.titles.remove(self.group.focused);
        self.lru.remove(self.group.focused);
        self.adjust_after_remove();
        true
    }

    pub fn close_tab_by_title(&mut self, title: &str) -> bool {
        let Some(idx) = self.titles.iter().position(|t| t == title) else {
            return false;
        };
        if let CloseResult::Denied(_) = self.group.children[idx].can_close() {
            return false;
        }
        self.group.children.remove(idx);
        self.titles.remove(idx);
        self.lru.remove(idx);
        self.adjust_after_remove();
        true
    }

    pub fn focus_tab_by_title(&mut self, title: &str) -> bool {
        if let Some(idx) = self.titles.iter().position(|t| t == title) {
            self.set_active(idx);
            true
        } else {
            false
        }
    }

    pub(crate) fn content_rect(&self) -> Rect {
        let b = self.group.view.bounds;
        Rect::new(b.x, b.y + 1, b.w, b.h.saturating_sub(1))
    }

    pub fn has_tab_starting_with(&self, prefix: &str) -> bool {
        self.titles.iter().any(|t| t.starts_with(prefix))
    }

    pub fn rename_active(&mut self, new_title: impl Into<String>) {
        if let Some(title) = self.titles.get_mut(self.group.focused) {
            *title = new_title.into();
            self.group.view.dirty = true;
        }
    }

    fn touch_lru(&mut self) {
        self.lru_counter += 1;
        if let Some(v) = self.lru.get_mut(self.group.focused) {
            *v = self.lru_counter;
        }
    }

    fn adjust_after_remove(&mut self) {
        if self.group.focused >= self.group.children.len() && self.group.focused > 0 {
            self.group.focused -= 1;
        }
        if !self.lru.is_empty() {
            let mru = self
                .lru
                .iter()
                .enumerate()
                .max_by_key(|(_, &v)| v)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.group.focused = mru;
        }
        if let Some(child) = self.group.children.get_mut(self.group.focused) {
            let b = self.group.view.bounds;
            let r = Rect::new(b.x, b.y + 1, b.w, b.h.saturating_sub(1));
            child.set_bounds(r);
            if self.group.view.focused {
                child.select();
            }
        }
        self.group.view.dirty = true;
    }
}

impl Default for TabGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tab_group_tests.rs"]
mod tests;
