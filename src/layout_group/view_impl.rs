//! View trait implementation for LayoutGroup.

use txv_core::prelude::*;

use super::{LayoutGroup, SlotId};

impl View for LayoutGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.set_bounds(r);
        self.group.view.mark_dirty();
        // Recompute proportional sizes on terminal resize
        // Wide: 1:2:2 by width. Tall: 1:2 width, 2:1 height.
        if r.w > 0 {
            let usable_w = r.w.saturating_sub(2); // max 2 dividers
            self.left_width = usable_w / 5;
            self.right_width = usable_w * 2 / 5;
        }
        if r.h > 0 {
            self.right_height = r.h * 2 / 3;
            self.bottom_height = r.h / 3;
        }
        self.apply_layout(r);
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.group.view.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        if let Some(z) = self.zoomed {
            // Zoomed panel is on top — draw it with chrome
            if let Some(c) = self.group.child(z) {
                c.draw(surface);
            }
            self.draw_zoomed_chrome(surface, z);
            return;
        }
        for child in self.group.children_iter() {
            let pb = child.bounds();
            if pb.w > 0 && pb.h > 0 {
                child.draw(surface);
            }
        }
        // Chrome overwrites TabGroup's plain chrome with Powerline visuals
        self.draw_chrome(surface);
        self.draw_dividers(surface, b);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Commands handled by LayoutGroup itself
        if let Event::Command { id, data } = event {
            let r = self.handle_command(*id, data, queue);
            if r == HandleResult::Consumed {
                return r;
            }
        }
        // Tick goes to ALL slots (background tabs need it for PTY poll, git refresh)
        if matches!(event, Event::Tick) {
            for child in self.group.children_iter_mut() {
                child.handle(event, queue);
            }
            return HandleResult::Ignored;
        }
        // All other events: delegate to focused child via GroupState 3-phase dispatch
        self.group.dispatch(event, queue)
    }
}

impl LayoutGroup {
    fn draw_dividers(&self, surface: &mut Surface, b: Rect) {
        let cs = Style {
            fg: Color::Ansi(7),
            bg: Color::Ansi(0),
            ..Style::default()
        };
        let rects = self.compute_rects(b);
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        // Vertical dividers (below chrome row)
        if left_r.w > 0 && center_r.w > 0 {
            let x = left_r.x + left_r.w;
            surface.vline(x, b.y + 1, left_r.h.saturating_sub(1), '│', cs);
        }
        if center_r.w > 0 && right_r.w > 0 {
            let x = right_r.x.saturating_sub(1);
            surface.vline(x, b.y + 1, right_r.h.saturating_sub(1), '│', cs);
        }
    }
}
