//! Layout computation for LayoutGroup.

use txv_core::prelude::*;

use super::{LayoutGroup, SlotId, PANEL_COUNT};

impl LayoutGroup {
    /// Compute and apply layout to all panels.
    pub(super) fn apply_layout(&mut self, bounds: Rect) {
        if let Some(z) = self.zoomed {
            self.group.children[z].set_bounds(bounds);
            return;
        }
        let rects = self.compute_rects(bounds);
        let tall = self.is_tall();
        self.was_tall = tall;
        for i in 0..PANEL_COUNT {
            let r = if tall && i == SlotId::Right as usize {
                rects[SlotId::Bottom as usize]
            } else if tall && i == SlotId::Bottom as usize {
                Rect::default()
            } else {
                rects[i]
            };
            self.group.children[i].set_bounds(r);
        }
    }

    pub(crate) fn compute_rects(&self, bounds: Rect) -> [Rect; PANEL_COUNT] {
        let mut rects = [Rect::default(); PANEL_COUNT];
        if bounds.w == 0 || bounds.h == 0 {
            return rects;
        }
        let tall = self.is_tall();
        let bottom_h = self.effective_bottom_height(bounds.h, tall);
        let div_h = u16::from(bottom_h > 0);
        let top_h = bounds.h.saturating_sub(bottom_h + div_h);

        self.fill_top(&mut rects, bounds, top_h, tall);

        if bottom_h > 0 {
            let y = bounds.y + top_h + div_h;
            rects[SlotId::Bottom as usize] = Rect::new(bounds.x, y, bounds.w, bottom_h);
        }
        rects
    }

    fn effective_bottom_height(&self, total_h: u16, tall: bool) -> u16 {
        let right_has = self.panel(SlotId::Right).tab_count() > 0;
        let bottom_has = self.panel(SlotId::Bottom).tab_count() > 0;
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
        let left_has = self.panel(SlotId::Left).tab_count() > 0;
        let right_has = self.panel(SlotId::Right).tab_count() > 0;

        let left_w = if left_has {
            self.left_width.min(bounds.w / 3)
        } else {
            0
        };
        let left_div = u16::from(left_w > 0);

        let (right_w, right_div) = if tall || !right_has {
            (0u16, 0u16)
        } else {
            (self.right_width.min(bounds.w / 2), 1u16)
        };

        let center_w = bounds.w.saturating_sub(left_w + left_div + right_w + right_div);

        let mut x = bounds.x;
        rects[SlotId::Left as usize] = Rect::new(x, bounds.y, left_w, h);
        x += left_w + left_div;
        rects[SlotId::Center as usize] = Rect::new(x, bounds.y, center_w, h);
        x += center_w + right_div;
        if !tall {
            rects[SlotId::Right as usize] = Rect::new(x, bounds.y, right_w, h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::LayoutMode;
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
        let bounds = Rect::new(0, 0, 200, 50);
        lg.panel_mut(SlotId::Center).insert_tab(
            "Editor",
            Box::new(Dv {
                state: ViewState::default(),
            }),
        );
        lg.zoomed = Some(1);
        lg.group.view.bounds = bounds;
        lg.apply_layout(bounds);
        // Zoomed panel gets full bounds
        assert_eq!(lg.group.children[1].bounds(), bounds);
    }

    #[test]
    fn wide_layout_three_columns() {
        let mut lg = LayoutGroup::new();
        lg.group.view.bounds = Rect::new(0, 0, 200, 50);
        lg.layout_mode = LayoutMode::Wide;
        lg.panel_mut(SlotId::Left).insert_tab(
            "Files",
            Box::new(Dv {
                state: ViewState::default(),
            }),
        );
        lg.panel_mut(SlotId::Center).insert_tab(
            "Editor",
            Box::new(Dv {
                state: ViewState::default(),
            }),
        );
        lg.panel_mut(SlotId::Right).insert_tab(
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
