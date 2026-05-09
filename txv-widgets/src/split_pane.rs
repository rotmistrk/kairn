//! SplitPane — two child views with a resizable divider.

use txv_core::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, // left | right
    Vertical,   // top / bottom
}

pub struct SplitPane {
    state: ViewState,
    pub direction: SplitDirection,
    pub ratio: f32, // 0.0..=1.0, position of divider
    pub first: Box<dyn View>,
    pub second: Box<dyn View>,
    pub focused_pane: usize, // 0 = first, 1 = second
}

impl SplitPane {
    pub fn new(
        direction: SplitDirection,
        first: Box<dyn View>,
        second: Box<dyn View>,
    ) -> Self {
        Self {
            state: ViewState::default(),
            direction,
            ratio: 0.5,
            first,
            second,
            focused_pane: 0,
        }
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.1, 0.9);
        self.layout();
    }

    pub fn resize(&mut self, delta: i16) {
        let total = match self.direction {
            SplitDirection::Horizontal => self.state.bounds.w,
            SplitDirection::Vertical => self.state.bounds.h,
        } as f32;
        if total > 0.0 {
            self.ratio = (self.ratio + delta as f32 / total).clamp(0.1, 0.9);
            self.layout();
        }
    }

    fn layout(&mut self) {
        let b = self.state.bounds;
        match self.direction {
            SplitDirection::Horizontal => {
                let split = (b.w as f32 * self.ratio) as u16;
                self.first.set_bounds(Rect::new(b.x, b.y, split, b.h));
                self.second.set_bounds(Rect::new(
                    b.x + split + 1,
                    b.y,
                    b.w.saturating_sub(split + 1),
                    b.h,
                ));
            }
            SplitDirection::Vertical => {
                let split = (b.h as f32 * self.ratio) as u16;
                self.first.set_bounds(Rect::new(b.x, b.y, b.w, split));
                self.second.set_bounds(Rect::new(
                    b.x,
                    b.y + split + 1,
                    b.w,
                    b.h.saturating_sub(split + 1),
                ));
            }
        }
        self.state.dirty = true;
    }
}

impl View for SplitPane {
    delegate_view_state!(state, override {
        set_bounds,
        select,
        unselect,
        needs_redraw,
        mark_redrawn,
    });

    fn set_bounds(&mut self, r: Rect) {
        self.state.bounds = r;
        self.state.dirty = true;
        self.layout();
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        self.first.draw(surface);
        self.second.draw(surface);

        // Draw divider
        let divider_style = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        match self.direction {
            SplitDirection::Horizontal => {
                let x = b.x + (b.w as f32 * self.ratio) as u16;
                surface.vline(x, b.y, b.h, '│', divider_style);
            }
            SplitDirection::Vertical => {
                let y = b.y + (b.h as f32 * self.ratio) as u16;
                surface.hline(b.x, y, b.w, '─', divider_style);
            }
        }
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        let target = if self.focused_pane == 0 {
            &mut self.first
        } else {
            &mut self.second
        };
        target.handle(event, queue)
    }

    fn select(&mut self) {
        self.state.focused = true;
        self.state.dirty = true;
        if self.focused_pane == 0 {
            self.first.select();
        } else {
            self.second.select();
        }
    }

    fn unselect(&mut self) {
        self.state.focused = false;
        self.state.dirty = true;
        self.first.unselect();
        self.second.unselect();
    }

    fn needs_redraw(&self) -> bool {
        self.state.dirty || self.first.needs_redraw() || self.second.needs_redraw()
    }

    fn mark_redrawn(&mut self) {
        self.state.dirty = false;
        self.first.mark_redrawn();
        self.second.mark_redrawn();
    }
}
