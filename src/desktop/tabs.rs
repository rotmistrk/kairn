//! Tab utility methods — naming, renaming, alt-digit selection.

use txv_core::prelude::*;

use super::{SlotId, SlottedDesktop};

impl SlottedDesktop {
    /// Find first available name like "Shell:0", "Shell:1", etc. for the given slot.
    pub fn next_tab_name(&self, slot: SlotId, prefix: &str) -> String {
        let s = &self.slots[slot as usize];
        for n in 0..10 {
            let candidate = format!("{prefix}:{n}");
            if !s.tabs.iter().any(|(t, _)| t.starts_with(&candidate)) {
                return candidate;
            }
        }
        format!("{prefix}:0")
    }

    /// Rename the active tab in the focused slot (tool tabs only).
    /// Preserves the type prefix (e.g., "Shell:") and replaces the user part.
    pub fn rename_focused_tab(&mut self, new_user_part: &str) {
        let s = &mut self.slots[self.focused as usize];
        if let Some((title, _)) = s.tabs.get_mut(s.active) {
            if let Some(colon) = title.find(':') {
                let prefix = &title[..=colon];
                *title = format!("{}{}", prefix, new_user_part);
                self.group.view.dirty = true;
            }
        }
    }

    /// M-0..9: select tab by index in the focused slot.
    pub(super) fn handle_alt_digit(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        use txv_core::event::KeyCode;
        if key.modifiers.alt && !key.modifiers.ctrl {
            if let KeyCode::Char(c) = &key.code {
                if c.is_ascii_digit() {
                    let idx = (*c as u8 - b'0') as usize;
                    let s = &self.slots[self.focused as usize];
                    if idx < s.tabs.len() {
                        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
                            v.unselect();
                        }
                        self.slots[self.focused as usize].active = idx;
                        self.sync_active_bounds(self.focused);
                        self.group.view.dirty = true;
                    }
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }

    /// Resize the focused slot by delta (positive = grow, negative = shrink).
    pub(super) fn resize_focused(&mut self, delta: i16) {
        let s = &mut self.slots[self.focused as usize];
        if delta > 0 {
            s.size += delta as u16;
        } else {
            s.size = s.size.saturating_sub((-delta) as u16);
        }
        self.set_bounds(self.group.view.bounds);
    }
}
