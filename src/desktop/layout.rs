//! Layout computation for SlottedDesktop.

use txv_core::prelude::*;
use super::{SlotId, SlottedDesktop, SLOT_COUNT};

impl SlottedDesktop {
    /// Compute inner rects for each slot given total bounds.
    pub(super) fn layout(&self, bounds: Rect) -> [Rect; SLOT_COUNT] {
        let mut rects = [Rect::default(); SLOT_COUNT];
        if bounds.w == 0 || bounds.h == 0 { return rects; }

        let chrome_h = 1u16;
        let content_y = bounds.y + chrome_h;

        let bottom = &self.slots[SlotId::Bottom as usize];
        let bottom_h = if bottom.visible && !bottom.tabs.is_empty() {
            bottom.size.min(bounds.h.saturating_sub(chrome_h + 2))
        } else { 0 };
        let bottom_divider = if bottom_h > 0 { 1u16 } else { 0 };

        let top_h = bounds.h.saturating_sub(chrome_h).saturating_sub(bottom_h).saturating_sub(bottom_divider);

        let left = &self.slots[SlotId::Left as usize];
        let right = &self.slots[SlotId::Right as usize];

        let left_w = if left.visible && !left.tabs.is_empty() { left.size.min(bounds.w / 3) } else { 0 };
        let left_div = if left_w > 0 { 1u16 } else { 0 };

        let right_w = if right.visible && !right.tabs.is_empty() { right.size.min(bounds.w / 3) } else { 0 };
        let right_div = if right_w > 0 { 1u16 } else { 0 };

        let center_w = bounds.w.saturating_sub(left_w).saturating_sub(left_div).saturating_sub(right_w).saturating_sub(right_div);

        if let Some(z) = self.zoomed {
            rects[z as usize] = Rect::new(bounds.x, content_y, bounds.w, bounds.h.saturating_sub(chrome_h));
            return rects;
        }

        let mut x = bounds.x;
        rects[SlotId::Left as usize] = Rect::new(x, content_y, left_w, top_h);
        x += left_w + left_div;
        rects[SlotId::Center as usize] = Rect::new(x, content_y, center_w, top_h);
        x += center_w + right_div;
        rects[SlotId::Right as usize] = Rect::new(x, content_y, right_w, top_h);

        let bottom_y = content_y + top_h + bottom_divider;
        rects[SlotId::Bottom as usize] = Rect::new(bounds.x, bottom_y, bounds.w, bottom_h);

        rects
    }
}
