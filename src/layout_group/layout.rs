//! Layout computation for LayoutGroup.

use txv_core::prelude::*;

use super::{LayoutGroup, PANEL_COUNT};
use crate::desktop::SlotId;

impl LayoutGroup {
    /// Compute and apply layout to all panels.
    pub(super) fn apply_layout(&mut self, bounds: Rect) {
        let rects = self.compute_rects(bounds);
        for (i, panel) in self.panels.iter_mut().enumerate() {
            panel.set_bounds(rects[i]);
        }
    }

    fn compute_rects(&self, bounds: Rect) -> [Rect; PANEL_COUNT] {
        let mut rects = [Rect::default(); PANEL_COUNT];
        if bounds.w == 0 || bounds.h == 0 {
            return rects;
        }

        if let Some(z) = self.zoomed {
            rects[z] = bounds;
            return rects;
        }

        let tall = self.is_tall();
        let bottom_h = self.effective_bottom_height(bounds.h, tall);
        let div_h = if bottom_h > 0 {
            1u16
        } else {
            0
        };
        let top_h = bounds.h.saturating_sub(bottom_h + div_h);

        self.fill_top(&mut rects, bounds, top_h, tall);

        if bottom_h > 0 {
            let y = bounds.y + top_h + div_h;
            rects[SlotId::Bottom as usize] = Rect::new(bounds.x, y, bounds.w, bottom_h);
        }
        rects
    }

    fn effective_bottom_height(&self, total_h: u16, tall: bool) -> u16 {
        let right_has = self.panels[SlotId::Right as usize].tab_count() > 0;
        let bottom_has = self.panels[SlotId::Bottom as usize].tab_count() > 0;
        let avail = total_h.saturating_sub(4);

        if tall && right_has {
            self.right_height.min(avail / 2)
        } else if bottom_has {
            self.bottom_height.min(avail)
        } else {
            0
        }
    }

    fn fill_top(&self, rects: &mut [Rect; PANEL_COUNT], bounds: Rect, h: u16, tall: bool) {
        let left_has = self.panels[SlotId::Left as usize].tab_count() > 0;
        let left_w = if left_has {
            self.left_width.min(bounds.w / 3)
        } else {
            0
        };
        let left_div = if left_w > 0 {
            1u16
        } else {
            0
        };

        let right_has = self.panels[SlotId::Right as usize].tab_count() > 0;
        let (right_w, right_div) = if tall || !right_has {
            (0u16, 0u16)
        } else {
            let rw = self.right_width.min(bounds.w / 3);
            (rw, 1u16)
        };

        let center_w = bounds.w.saturating_sub(left_w + left_div + right_w + right_div);

        let mut x = bounds.x;
        rects[SlotId::Left as usize] = Rect::new(x, bounds.y, left_w, h);
        x += left_w + left_div;
        rects[SlotId::Center as usize] = Rect::new(x, bounds.y, center_w, h);
        x += center_w + right_div;
        if !tall {
            rects[SlotId::Right as usize] = Rect::new(x, bounds.y, right_w, h);
        } else if right_has {
            // In tall mode, right panel goes below center (uses bottom area)
            // Already handled by effective_bottom_height putting right in bottom slot
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dv {
        state: ViewState,
    }
    impl View for Dv {
        delegate_view_state!(state);
        fn draw(&self, _: &mut Surface) {}
        fn handle(&mut self, _: &Event, _: &mut EventQueue) -> HandleResult {
            HandleResult::Ignored
        }
    }

    #[test]
    fn empty_panels_get_zero_rects() {
        let lg = LayoutGroup::new();
        let rects = lg.compute_rects(Rect::new(0, 0, 200, 50));
        assert_eq!(rects[0].w, 0);
        assert_eq!(rects[2].w, 0);
    }

    #[test]
    fn zoom_gives_full_bounds() {
        let mut lg = LayoutGroup::new();
        lg.zoomed = Some(1);
        let bounds = Rect::new(0, 0, 200, 50);
        let rects = lg.compute_rects(bounds);
        assert_eq!(rects[1], bounds);
        assert_eq!(rects[0], Rect::default());
    }

    #[test]
    fn wide_layout_three_columns() {
        let mut lg = LayoutGroup::new();
        lg.state.bounds = Rect::new(0, 0, 200, 50);
        lg.layout_mode = crate::layout_group::LayoutMode::Wide;
        lg.panels[0].insert_tab(
            "Files",
            Box::new(Dv {
                state: ViewState::default(),
            }),
        );
        lg.panels[1].insert_tab(
            "Editor",
            Box::new(Dv {
                state: ViewState::default(),
            }),
        );
        lg.panels[2].insert_tab(
            "Shell",
            Box::new(Dv {
                state: ViewState::default(),
            }),
        );

        let rects = lg.compute_rects(Rect::new(0, 0, 200, 50));
        assert!(rects[0].w > 0, "left should have width");
        assert!(rects[1].w > 0, "center should have width");
        assert!(rects[2].w > 0, "right should have width");
        assert_eq!(rects[0].w + 1 + rects[1].w + 1 + rects[2].w, 200);
    }
}
