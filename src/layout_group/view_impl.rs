//! View trait implementation for LayoutGroup.

use txv_core::prelude::*;

use super::{LayoutGroup, SlotId};

impl View for LayoutGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.group.mark_dirty();
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

    fn draw(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        self.group.buffer_mut().fill(' ', Style::default());
        let my_bounds = self.group.bounds();
        if let Some(z) = self.zoomed {
            // Zoomed: chrome background line, then panel renders its own TabBar
            let cs = txv_core::palette::palette().chrome.bar.to_style();
            self.group.buffer_mut().hline(0, 0, w, '─', cs);
            if let Some(c) = self.group.child_mut(z) {
                c.draw();
            }
            let cb = self.group.child(z).map(|c| c.bounds()).unwrap_or_default();
            let buf_ptr = self.group.buffer_mut() as *mut Buffer;
            if let Some(c) = self.group.child(z) {
                let dx = cb.x.saturating_sub(my_bounds.x);
                let dy = cb.y.saturating_sub(my_bounds.y);
                unsafe { (*buf_ptr).blit(c.buffer(), dx, dy) };
            }
            return;
        }
        // Draw chrome background: horizontal lines + connectors
        self.draw_chrome_background();
        // Draw all children (TabPanels render their own TabBar with transparent fill)
        for child in self.group.children_iter_mut() {
            let pb = child.bounds();
            if pb.w > 0 && pb.h > 0 {
                child.draw();
            }
        }
        // Blit all children
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child(i) {
                let cb = child.bounds();
                if cb.w > 0 && cb.h > 0 {
                    let dx = cb.x.saturating_sub(my_bounds.x);
                    let dy = cb.y.saturating_sub(my_bounds.y);
                    unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
                }
            }
        }
        self.draw_dividers();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Commands handled by LayoutGroup itself
        if let Event::Command { id, data } = event {
            let r = self.handle_command(*id, data);
            if r == HandleResult::Consumed {
                return r;
            }
        }
        // Tick goes to ALL slots (background tabs need it for PTY poll, git refresh)
        if matches!(event, Event::Tick) {
            for child in self.group.children_iter_mut() {
                child.handle(event);
            }
            return HandleResult::Ignored;
        }
        // All other events: delegate to focused child via GroupState 3-phase dispatch
        self.group.dispatch(event)
    }
}

impl LayoutGroup {
    fn draw_dividers(&mut self) {
        let b = self.group.bounds();
        let cs = txv_core::palette::palette().chrome.bar.to_style();
        let rects = self.compute_rects(b);
        let left_r = rects[SlotId::Left as usize];
        let center_r = rects[SlotId::Center as usize];
        let right_r = rects[SlotId::Right as usize];
        // Vertical dividers (below chrome row)
        if left_r.w > 0 && center_r.w > 0 {
            let x = (left_r.x + left_r.w).saturating_sub(b.x);
            self.group.buffer_mut().vline(x, 1, left_r.h.saturating_sub(1), '│', cs);
        }
        if center_r.w > 0 && right_r.w > 0 {
            let x = right_r.x.saturating_sub(1).saturating_sub(b.x);
            self.group
                .buffer_mut()
                .vline(x, 1, right_r.h.saturating_sub(1), '│', cs);
        }
    }
}
