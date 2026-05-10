//! Layout computation for SlottedDesktop.

use txv_core::prelude::*;
use super::{LayoutMode, SlotId, SlottedDesktop, SLOT_COUNT};

const WIDE_THRESHOLD: u16 = 200;

impl SlottedDesktop {
    pub(super) fn is_tall(&self, width: u16) -> bool {
        match self.layout_mode {
            LayoutMode::Wide => false,
            LayoutMode::Tall => true,
            LayoutMode::Auto => width < WIDE_THRESHOLD,
        }
    }

    /// Compute inner rects for each slot given total bounds.
    pub(super) fn layout(&self, bounds: Rect) -> [Rect; SLOT_COUNT] {
        let mut rects = [Rect::default(); SLOT_COUNT];
        if bounds.w == 0 || bounds.h == 0 { return rects; }

        let chrome_h = 1u16;
        let content_y = bounds.y + chrome_h;

        if let Some(z) = self.zoomed {
            rects[z as usize] = Rect::new(
                bounds.x, content_y, bounds.w, bounds.h.saturating_sub(chrome_h),
            );
            return rects;
        }

        let tall = self.is_tall(bounds.w);
        let panel_h = self.compute_panel_height(bounds.h, chrome_h, tall);
        let panel_div = if panel_h > 0 { 1u16 } else { 0 };
        let top_h = bounds.h.saturating_sub(chrome_h + panel_h + panel_div);

        self.fill_top_slots(&mut rects, bounds, content_y, top_h, tall);

        let bottom_y = content_y + top_h + panel_div;
        rects[SlotId::Bottom as usize] = Rect::new(bounds.x, bottom_y, bounds.w, panel_h);
        rects
    }

    fn compute_panel_height(&self, total_h: u16, chrome_h: u16, tall: bool) -> u16 {
        let right = &self.slots[SlotId::Right as usize];
        let right_has = right.visible && !right.tabs.is_empty();
        let bottom = &self.slots[SlotId::Bottom as usize];
        let bottom_has = bottom.visible && !bottom.tabs.is_empty();

        let avail = total_h.saturating_sub(chrome_h + 2);
        if tall && right_has {
            right.size.min(avail / 2)
        } else if bottom_has {
            bottom.size.min(avail)
        } else {
            0
        }
    }

    fn fill_top_slots(
        &self,
        rects: &mut [Rect; SLOT_COUNT],
        bounds: Rect,
        y: u16,
        h: u16,
        tall: bool,
    ) {
        let left = &self.slots[SlotId::Left as usize];
        let left_w = if left.visible && !left.tabs.is_empty() {
            left.size.min(bounds.w / 3)
        } else { 0 };
        let left_div = if left_w > 0 { 1u16 } else { 0 };

        let right = &self.slots[SlotId::Right as usize];
        let right_has = right.visible && !right.tabs.is_empty();
        let (right_w, right_div) = if tall {
            (0u16, 0u16)
        } else {
            let rw = if right_has { right.size.min(bounds.w / 3) } else { 0 };
            (rw, if rw > 0 { 1u16 } else { 0 })
        };

        let center_w = bounds.w.saturating_sub(left_w + left_div + right_w + right_div);

        let mut x = bounds.x;
        rects[SlotId::Left as usize] = Rect::new(x, y, left_w, h);
        x += left_w + left_div;
        rects[SlotId::Center as usize] = Rect::new(x, y, center_w, h);
        x += center_w + right_div;
        rects[SlotId::Right as usize] = Rect::new(x, y, right_w, h);
    }
}
