//! View trait implementation and chrome drawing for TabGroup.

use txv_core::prelude::*;

use super::tab_group::TabGroup;

impl TabGroup {
    pub(crate) fn draw_chrome(&self, surface: &mut Surface) {
        let b = self.group.view.bounds;
        if b.w == 0 || b.h == 0 || self.titles.is_empty() {
            return;
        }
        let dim = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let bright = Style {
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        surface.hline(b.x, b.y, b.w, '─', dim);
        let mut x = b.x;
        for (i, title) in self.titles.iter().enumerate() {
            let style = if i == self.group.focused {
                bright
            } else {
                dim
            };
            let label = format!(" {title} ");
            let len = label.len() as u16;
            if x + len > b.x + b.w {
                break;
            }
            surface.print(x, b.y, &label, style);
            x += len;
        }
        if self.titles.len() > 1 {
            let count = format!("❨{}❩", self.titles.len());
            let clen = count.chars().count() as u16;
            if x + clen < b.x + b.w {
                surface.print(x + 1, b.y, &count, dim);
            }
        }
    }
}

impl View for TabGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        self.group.view.dirty = true;
        let content = self.content_rect();
        if let Some(child) = self.group.children.get_mut(self.group.focused) {
            child.set_bounds(content);
        }
    }

    fn draw(&self, surface: &mut Surface) {
        self.draw_chrome(surface);
        if let Some(child) = self.group.children.get(self.group.focused) {
            child.draw(surface);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Dispatch to focused child (active tab) via GroupState 3-phase dispatch
        self.group.dispatch(event, queue)
    }
}
