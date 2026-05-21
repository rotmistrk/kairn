//! Terminal activity badge tracking.

use std::time::Instant;

use super::{LayoutGroup, SlotId, TabBadge};

impl LayoutGroup {
    /// Update terminal activity badges. Call from tick handler.
    /// `idle_secs` is the threshold before a terminal is considered idle.
    pub fn update_badges(&mut self, idle_secs: u64) -> Vec<String> {
        let now = Instant::now();
        let idle_dur = std::time::Duration::from_secs(idle_secs);
        let mut auto_close = Vec::new();

        // Collect tab info first to avoid borrow conflicts
        let mut tab_info: Vec<(SlotId, usize, String, bool)> = Vec::new();
        for slot in [SlotId::Right, SlotId::Bottom] {
            let panel = self.panel(slot);
            for i in 0..panel.tab_count() {
                let title = panel.tab_title(i).unwrap_or_default().to_string();
                let dirty = panel.view_at(i).is_some_and(|v| v.needs_redraw());
                tab_info.push((slot, i, title, dirty));
            }
        }

        for (slot, i, title, dirty) in tab_info {
            let key = (slot, i);
            if title.contains("[exited]") {
                self.badges.insert(key, TabBadge::Exited);
                self.last_output.remove(&key);
                auto_close.push(title);
            } else if dirty {
                self.last_output.insert(key, now);
                self.badges.insert(key, TabBadge::Busy);
            } else {
                let last = self.last_output.get(&key).copied().unwrap_or(now);
                if now.duration_since(last) > idle_dur {
                    self.badges.insert(key, TabBadge::Idle);
                } else {
                    self.badges.insert(key, TabBadge::Busy);
                }
            }
        }
        auto_close
    }

    /// Get the badge for the active tab in a slot.
    pub fn active_badge(&self, slot: SlotId) -> Option<TabBadge> {
        let idx = self.panel(slot).active_index();
        self.badges.get(&(slot, idx)).copied()
    }
}
