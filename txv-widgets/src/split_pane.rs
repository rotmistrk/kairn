//! SplitPane — two child views with a resizable divider.
//! Uses GroupState: children[0] = first, children[1] = second.

use txv_core::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, // left | right
    Vertical,   // top / bottom
}

pub struct SplitPane {
    group: GroupState,
    pub direction: SplitDirection,
    pub ratio: f32, // 0.0..=1.0, position of divider
}

impl SplitPane {
    pub fn new(direction: SplitDirection, first: Box<dyn View>, second: Box<dyn View>) -> Self {
        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        group.insert(first);
        group.insert(second);
        Self {
            group,
            direction,
            ratio: 0.5,
        }
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.1, 0.9);
        self.apply_layout();
    }

    pub fn resize(&mut self, delta: i16) {
        let total = match self.direction {
            SplitDirection::Horizontal => self.group.view.bounds().w,
            SplitDirection::Vertical => self.group.view.bounds().h,
        } as f32;
        if total > 0.0 {
            self.ratio = (self.ratio + delta as f32 / total).clamp(0.1, 0.9);
            self.apply_layout();
        }
    }

    fn apply_layout(&mut self) {
        let b = self.group.view.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let (r1, r2) = match self.direction {
            SplitDirection::Horizontal => {
                let split = (b.w as f32 * self.ratio) as u16;
                (
                    Rect::new(b.x, b.y, split, b.h),
                    Rect::new(b.x + split + 1, b.y, b.w.saturating_sub(split + 1), b.h),
                )
            }
            SplitDirection::Vertical => {
                let split = (b.h as f32 * self.ratio) as u16;
                (
                    Rect::new(b.x, b.y, b.w, split),
                    Rect::new(b.x, b.y + split + 1, b.w, b.h.saturating_sub(split + 1)),
                )
            }
        };
        self.group.set_child_bounds(0, r1);
        self.group.set_child_bounds(1, r2);
    }
}

impl View for SplitPane {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.set_bounds(r);
        self.group.view.mark_dirty();
        self.apply_layout();
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.group.view.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        for child in self.group.children_iter() {
            child.draw(surface);
        }
        let dim = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        match self.direction {
            SplitDirection::Horizontal => {
                let x = b.x + (b.w as f32 * self.ratio) as u16;
                surface.vline(x, b.y, b.h, '│', dim);
            }
            SplitDirection::Vertical => {
                let y = b.y + (b.h as f32 * self.ratio) as u16;
                surface.hline(b.x, y, b.w, '─', dim);
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        self.group.dispatch(event, queue)
    }
}
