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
        if let Some(child) = self.group.focused_child_mut() {
            child.unselect();
        }
        self.group.insert(view);
        self.titles.push(title.into());
        self.lru.push(0);
        self.group.set_focused_index(self.group.child_count() - 1);
        self.touch_lru();
        if self.group.view.is_focused() {
            if let Some(child) = self.group.focused_child_mut() {
                child.select();
            }
        }
        self.group.view.mark_dirty();
    }

    pub fn tab_count(&self) -> usize {
        self.group.child_count()
    }

    pub fn set_active(&mut self, index: usize) {
        if index >= self.group.child_count() || index == self.group.focused_index() {
            return;
        }
        self.group.unselect_focused();
        self.group.set_focused_index(index);
        self.touch_lru();
        let r = self.content_rect();
        self.group.set_child_bounds(self.group.focused_index(), r);
        if self.group.view.is_focused() {
            self.group.select_focused();
        }
        self.group.view.mark_dirty();
    }

    pub fn active_title(&self) -> Option<&str> {
        self.titles.get(self.group.focused_index()).map(|t| t.as_str())
    }

    /// Index of the currently active tab.
    pub fn active_index(&self) -> usize {
        self.group.focused_index()
    }

    pub fn tab_title(&self, index: usize) -> Option<&str> {
        self.titles.get(index).map(|t| t.as_str())
    }

    pub fn active_view_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.group.focused_child_mut()
    }

    /// Access a child view by index.
    pub fn view_at(&self, index: usize) -> Option<&dyn View> {
        self.group.child(index)
    }

    /// Mutable access to a child view by index.
    pub fn view_at_mut(&mut self, index: usize) -> Option<&mut Box<dyn View>> {
        self.group.child_mut(index)
    }

    pub fn tab_next(&mut self) {
        if self.group.child_count() > 1 {
            self.set_active((self.group.focused_index() + 1) % self.group.child_count());
        }
    }

    pub fn tab_prev(&mut self) {
        if self.group.child_count() > 1 {
            let prev = if self.group.focused_index() == 0 {
                self.group.child_count() - 1
            } else {
                self.group.focused_index() - 1
            };
            self.set_active(prev);
        }
    }

    pub fn close_active(&mut self) -> bool {
        if self.group.is_empty() {
            return false;
        }
        let fi = self.group.focused_index();
        if let Some(child) = self.group.child_mut(fi) {
            if let CloseResult::Denied(_) = child.can_close() {
                return false;
            }
        }
        self.group.remove(fi);
        self.titles.remove(fi);
        self.lru.remove(fi);
        self.adjust_after_remove();
        true
    }

    pub fn close_tab_by_title(&mut self, title: &str) -> bool {
        let Some(idx) = self.titles.iter().position(|t| t == title) else {
            return false;
        };
        if let Some(child) = self.group.child_mut(idx) {
            if let CloseResult::Denied(_) = child.can_close() {
                return false;
            }
        }
        self.group.remove(idx);
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
        let b = self.group.view.bounds();
        Rect::new(b.x, b.y + 1, b.w, b.h.saturating_sub(1))
    }

    pub fn has_tab_starting_with(&self, prefix: &str) -> bool {
        self.titles.iter().any(|t| t.starts_with(prefix))
    }

    pub fn rename_active(&mut self, new_title: impl Into<String>) {
        if let Some(title) = self.titles.get_mut(self.group.focused_index()) {
            *title = new_title.into();
            self.group.view.mark_dirty();
        }
    }

    /// Generate next available name like "Shell:0", "Shell:1", etc.
    pub fn next_tab_name(&self, prefix: &str) -> String {
        for n in 0..10 {
            let candidate = format!("{prefix}:{n}");
            if !self.has_tab_starting_with(&candidate) {
                return candidate;
            }
        }
        format!("{prefix}:0")
    }

    /// Rename active tab, keeping the "prefix:" part and replacing the user part.
    pub fn rename_user_part(&mut self, new_user_part: &str) {
        if let Some(title) = self.titles.get(self.group.focused_index()).cloned() {
            if let Some(colon) = title.find(':') {
                let prefix = &title[..=colon];
                self.rename_active(format!("{prefix}{new_user_part}"));
            }
        }
    }

    fn touch_lru(&mut self) {
        self.lru_counter += 1;
        if let Some(v) = self.lru.get_mut(self.group.focused_index()) {
            *v = self.lru_counter;
        }
    }

    fn adjust_after_remove(&mut self) {
        let fi = self.group.focused_index();
        if fi >= self.group.child_count() && fi > 0 {
            self.group.set_focused_index(fi - 1);
        }
        if !self.lru.is_empty() {
            let mru = self
                .lru
                .iter()
                .enumerate()
                .max_by_key(|(_, &v)| v)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.group.set_focused_index(mru);
        }
        let r = self.content_rect();
        let is_focused = self.group.view.is_focused();
        if let Some(child) = self.group.focused_child_mut() {
            child.set_bounds(r);
            if is_focused {
                child.select();
            }
        }
        self.group.view.mark_dirty();
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
