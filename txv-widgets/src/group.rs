//! Group — a View that contains and manages child Views.
//!
//! Replaces `FocusGroup`. Dispatches events to focused child first,
//! then others, then handles group-level commands itself.

use txv::layout::{Constraint, Direction, Rect};
use txv::surface::Surface;

use crate::view::{commands, DrawContext, Event, GrowFlags, HandleResult, View};

/// How a Group arranges its children.
#[derive(Clone, Debug)]
pub enum Layout {
    /// Split horizontally or vertically with constraints.
    Split {
        direction: Direction,
        constraints: Vec<Constraint>,
    },
    /// Stack: only one child visible at a time (tabs).
    Stack,
    /// Manual: children have fixed positions (dialogs, overlays).
    Manual,
}

/// A View that contains and manages child Views.
pub struct Group {
    bounds: Rect,
    children: Vec<Box<dyn View>>,
    focused: usize,
    layout: Layout,
}

impl Group {
    /// Create a new group with the given layout.
    pub fn new(layout: Layout) -> Self {
        Self {
            bounds: Rect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
            children: Vec::new(),
            focused: 0,
            layout,
        }
    }

    /// Add a child view.
    pub fn add(&mut self, child: Box<dyn View>) {
        self.children.push(child);
        self.relayout();
    }

    /// Remove a child by index.
    pub fn remove(&mut self, index: usize) -> Option<Box<dyn View>> {
        if index >= self.children.len() {
            return None;
        }
        let child = self.children.remove(index);
        if self.focused >= self.children.len() && !self.children.is_empty() {
            self.focused = self.children.len() - 1;
        }
        self.relayout();
        Some(child)
    }

    /// Move focus to the next focusable child.
    pub fn focus_next(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let start = self.focused;
        loop {
            self.focused = (self.focused + 1) % self.children.len();
            if self.children[self.focused].focusable() || self.focused == start {
                break;
            }
        }
    }

    /// Move focus to the previous focusable child.
    pub fn focus_prev(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let start = self.focused;
        loop {
            self.focused = if self.focused == 0 {
                self.children.len() - 1
            } else {
                self.focused - 1
            };
            if self.children[self.focused].focusable() || self.focused == start {
                break;
            }
        }
    }

    /// Index of the currently focused child.
    pub fn focused_index(&self) -> usize {
        self.focused
    }

    /// Number of children.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Get a reference to the focused child.
    pub fn focused_child(&self) -> Option<&dyn View> {
        self.children.get(self.focused).map(|c| c.as_ref())
    }

    /// Recompute child bounds from layout and own bounds.
    fn relayout(&mut self) {
        let rects = self.compute_child_rects();
        for (i, child) in self.children.iter_mut().enumerate() {
            if let Some(&r) = rects.get(i) {
                child.set_bounds(r);
            }
        }
    }

    /// Compute child rects based on layout.
    fn compute_child_rects(&self) -> Vec<Rect> {
        match &self.layout {
            Layout::Split {
                direction,
                constraints,
            } => self.bounds.split(*direction, constraints),
            Layout::Stack => {
                // All children get the full bounds.
                vec![self.bounds; self.children.len()]
            }
            Layout::Manual => {
                // Children keep their existing bounds.
                self.children.iter().map(|c| c.bounds()).collect()
            }
        }
    }

    /// Handle group-level commands (focus cycling).
    fn handle_self(&mut self, event: &Event) -> HandleResult {
        if let Event::Command(cmd) = event {
            match *cmd {
                commands::CM_FOCUS_NEXT => {
                    self.focus_next();
                    return HandleResult::Consumed;
                }
                commands::CM_FOCUS_PREV => {
                    self.focus_prev();
                    return HandleResult::Consumed;
                }
                _ => {}
            }
        }
        HandleResult::Ignored
    }
}

impl View for Group {
    fn draw(&self, surface: &mut Surface<'_>, ctx: &DrawContext) {
        let rects = self.compute_child_rects();
        for (i, child) in self.children.iter().enumerate() {
            if let Some(r) = rects.get(i) {
                // For Stack layout, only draw the focused child.
                if matches!(self.layout, Layout::Stack) && i != self.focused {
                    continue;
                }
                let rel_x = r.x.saturating_sub(self.bounds.x);
                let rel_y = r.y.saturating_sub(self.bounds.y);
                let mut sub = surface.sub(rel_x, rel_y, r.w, r.h);
                child.draw(&mut sub, ctx);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // 1. Try focused child first.
        if let Some(child) = self.children.get_mut(self.focused) {
            if child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }
        // 2. Try other children.
        for (i, child) in self.children.iter_mut().enumerate() {
            if i == self.focused {
                continue;
            }
            if child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }
        // 3. Handle ourselves (group-level commands).
        self.handle_self(event)
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
        self.relayout();
    }

    fn focusable(&self) -> bool {
        self.children.iter().any(|c| c.focusable())
    }

    fn grow_flags(&self) -> GrowFlags {
        self.children
            .iter()
            .fold(GrowFlags::NONE, |acc, c| acc.union(c.grow_flags()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::{DrawContext, Event, HandleResult, View};
    use txv::cell::Style;
    use txv::layout::{Constraint, Direction, Rect, Size};
    use txv::screen::Screen;

    /// Minimal test view.
    struct Stub {
        bounds: Rect,
        focus: bool,
        label: char,
    }

    impl Stub {
        fn new(label: char, focusable: bool) -> Self {
            Self {
                bounds: Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                },
                focus: focusable,
                label,
            }
        }
    }

    impl View for Stub {
        fn draw(&self, surface: &mut Surface<'_>, _ctx: &DrawContext) {
            if surface.width() > 0 && surface.height() > 0 {
                surface.put(0, 0, self.label, Style::default());
            }
        }
        fn handle(&mut self, event: &Event) -> HandleResult {
            match event {
                Event::Key(_) => HandleResult::Ignored,
                _ => HandleResult::Ignored,
            }
        }
        fn bounds(&self) -> Rect {
            self.bounds
        }
        fn set_bounds(&mut self, rect: Rect) {
            self.bounds = rect;
        }
        fn focusable(&self) -> bool {
            self.focus
        }
    }

    unsafe impl Send for Stub {}

    fn ctx() -> DrawContext {
        DrawContext {
            app_focused: true,
            tick: 0,
        }
    }

    #[test]
    fn empty_group() {
        let g = Group::new(Layout::Stack);
        assert_eq!(g.child_count(), 0);
        assert_eq!(g.focused_index(), 0);
        assert!(g.focused_child().is_none());
    }

    #[test]
    fn add_and_focus_next() {
        let mut g = Group::new(Layout::Stack);
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        g.add(Box::new(Stub::new('C', true)));
        assert_eq!(g.focused_index(), 0);
        g.focus_next();
        assert_eq!(g.focused_index(), 1);
        g.focus_next();
        assert_eq!(g.focused_index(), 2);
        g.focus_next();
        assert_eq!(g.focused_index(), 0); // wraps
    }

    #[test]
    fn focus_skips_unfocusable() {
        let mut g = Group::new(Layout::Stack);
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', false)));
        g.add(Box::new(Stub::new('C', true)));
        g.focus_next();
        assert_eq!(g.focused_index(), 2); // skipped B
    }

    #[test]
    fn focus_prev() {
        let mut g = Group::new(Layout::Stack);
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        g.focus_prev();
        assert_eq!(g.focused_index(), 1); // wraps backward
    }

    #[test]
    fn remove_adjusts_focus() {
        let mut g = Group::new(Layout::Stack);
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        g.focused = 1;
        g.remove(1);
        assert_eq!(g.focused_index(), 0);
    }

    #[test]
    fn handle_focus_commands() {
        let mut g = Group::new(Layout::Stack);
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        let r = g.handle(&Event::Command(commands::CM_FOCUS_NEXT));
        assert_eq!(r, HandleResult::Consumed);
        assert_eq!(g.focused_index(), 1);
        let r = g.handle(&Event::Command(commands::CM_FOCUS_PREV));
        assert_eq!(r, HandleResult::Consumed);
        assert_eq!(g.focused_index(), 0);
    }

    #[test]
    fn split_layout_sets_child_bounds() {
        let mut g = Group::new(Layout::Split {
            direction: Direction::Horizontal,
            constraints: vec![
                Constraint {
                    size: Size::Fixed(10),
                    min: 0,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
            ],
        });
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        g.set_bounds(Rect {
            x: 0,
            y: 0,
            w: 80,
            h: 24,
        });
        assert_eq!(g.children[0].bounds().w, 10);
        assert_eq!(g.children[1].bounds().w, 70);
    }

    #[test]
    fn draw_split() {
        let mut g = Group::new(Layout::Split {
            direction: Direction::Horizontal,
            constraints: vec![
                Constraint {
                    size: Size::Fixed(5),
                    min: 0,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
            ],
        });
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        g.set_bounds(Rect {
            x: 0,
            y: 0,
            w: 20,
            h: 5,
        });

        let mut screen = Screen::new(20, 5);
        {
            let mut s = screen.full_surface();
            g.draw(&mut s, &ctx());
        }
        assert_eq!(screen.cell(0, 0).ch, 'A');
        assert_eq!(screen.cell(5, 0).ch, 'B');
    }

    #[test]
    fn stack_draws_only_focused() {
        let mut g = Group::new(Layout::Stack);
        g.add(Box::new(Stub::new('A', true)));
        g.add(Box::new(Stub::new('B', true)));
        g.set_bounds(Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        });
        g.focused = 1;

        let mut screen = Screen::new(10, 5);
        {
            let mut s = screen.full_surface();
            g.draw(&mut s, &ctx());
        }
        assert_eq!(screen.cell(0, 0).ch, 'B');
    }
}
