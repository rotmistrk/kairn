//! Positioned popup container that wraps another widget.

use crossterm::event::KeyEvent;
use txv::border::{draw_border, BorderMode, BorderStyle};
use txv::cell::Style;
use txv::layout::Rect;
use txv::surface::Surface;

use crate::widget::{EventResult, Widget};

/// Anchor point for overlay positioning.
pub enum Anchor {
    /// Centered on screen.
    Center,
    /// Below a given (col, row) position.
    Below(u16, u16),
    /// Above a given (col, row) position.
    Above(u16, u16),
}

/// A positioned popup that wraps an inner widget with a border.
pub struct Overlay<W: Widget> {
    /// The inner widget.
    pub inner: W,
    anchor: Anchor,
    width: u16,
    height: u16,
    /// Border style for the overlay.
    pub border_style: BorderStyle,
}

impl<W: Widget> Overlay<W> {
    /// Create a new overlay wrapping the given widget.
    pub fn new(inner: W, anchor: Anchor, width: u16, height: u16) -> Self {
        Self {
            inner,
            anchor,
            width,
            height,
            border_style: BorderStyle {
                mode: BorderMode::Pretty,
                active: Style::default(),
                inactive: Style::default(),
            },
        }
    }

    /// Compute the overlay rectangle given screen dimensions.
    pub fn compute_rect(&self, screen_w: u16, screen_h: u16) -> Rect {
        let w = self.width.min(screen_w);
        let h = self.height.min(screen_h);
        let (x, y) = match self.anchor {
            Anchor::Center => {
                let x = screen_w.saturating_sub(w) / 2;
                let y = screen_h.saturating_sub(h) / 2;
                (x, y)
            }
            Anchor::Below(col, row) => {
                let x = col.min(screen_w.saturating_sub(w));
                let y = (row + 1).min(screen_h.saturating_sub(h));
                (x, y)
            }
            Anchor::Above(col, row) => {
                let x = col.min(screen_w.saturating_sub(w));
                let y = row.saturating_sub(h);
                (x, y)
            }
        };
        Rect { x, y, w, h }
    }
}

impl<W: Widget> Widget for Overlay<W> {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        let sw = surface.width();
        let sh = surface.height();
        let rect = self.compute_rect(sw, sh);

        // Clear the overlay area
        let mut area = surface.sub(rect.x, rect.y, rect.w, rect.h);
        area.fill(' ', Style::default());

        // Draw border
        let border_rect = Rect {
            x: 0,
            y: 0,
            w: rect.w,
            h: rect.h,
        };
        let inner = draw_border(&mut area, border_rect, "", &self.border_style, focused);

        // Render inner widget
        if inner.w > 0 && inner.h > 0 {
            let mut inner_surface = area.sub(inner.x, inner.y, inner.w, inner.h);
            self.inner.render(&mut inner_surface, focused);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        self.inner.handle_key(key)
    }

    fn focusable(&self) -> bool {
        self.inner.focusable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::WidgetAction;
    use crossterm::event::{KeyCode, KeyModifiers};

    /// Minimal test widget.
    struct TestWidget {
        label: String,
    }

    impl Widget for TestWidget {
        fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
            surface.print(0, 0, &self.label, Style::default());
        }

        fn handle_key(&mut self, key: KeyEvent) -> EventResult {
            match key.code {
                KeyCode::Enter => EventResult::Action(WidgetAction::Confirmed("ok".into())),
                _ => EventResult::Ignored,
            }
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn center_positioning() {
        let overlay = Overlay::new(TestWidget { label: "X".into() }, Anchor::Center, 20, 10);
        let rect = overlay.compute_rect(80, 24);
        assert_eq!(rect.x, 30);
        assert_eq!(rect.y, 7);
        assert_eq!(rect.w, 20);
        assert_eq!(rect.h, 10);
    }

    #[test]
    fn below_positioning() {
        let overlay = Overlay::new(TestWidget { label: "X".into() }, Anchor::Below(5, 3), 10, 5);
        let rect = overlay.compute_rect(80, 24);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 4); // row + 1
        assert_eq!(rect.w, 10);
        assert_eq!(rect.h, 5);
    }

    #[test]
    fn above_positioning() {
        let overlay = Overlay::new(
            TestWidget { label: "X".into() },
            Anchor::Above(5, 10),
            10,
            5,
        );
        let rect = overlay.compute_rect(80, 24);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 5); // row - height
        assert_eq!(rect.w, 10);
        assert_eq!(rect.h, 5);
    }

    #[test]
    fn clamp_to_screen() {
        let overlay = Overlay::new(TestWidget { label: "X".into() }, Anchor::Center, 100, 50);
        let rect = overlay.compute_rect(40, 20);
        assert_eq!(rect.w, 40);
        assert_eq!(rect.h, 20);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
    }

    #[test]
    fn below_clamps_right_edge() {
        let overlay = Overlay::new(
            TestWidget { label: "X".into() },
            Anchor::Below(75, 0),
            10,
            5,
        );
        let rect = overlay.compute_rect(80, 24);
        assert_eq!(rect.x, 70); // clamped so popup fits
    }

    #[test]
    fn delegates_key_to_inner() {
        let mut overlay = Overlay::new(TestWidget { label: "X".into() }, Anchor::Center, 20, 10);
        let result = overlay.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "ok"
        ));
    }

    #[test]
    fn render_draws_border_and_inner() {
        use txv::cell::ColorMode;
        use txv::screen::Screen;

        let overlay = Overlay::new(TestWidget { label: "HI".into() }, Anchor::Center, 10, 5);
        let mut screen = Screen::with_color_mode(20, 10, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            overlay.render(&mut s, true);
        }
        let text = screen.to_text();
        assert!(text.contains("HI"));
        assert!(text.contains('┌'));
    }

    #[test]
    fn focusable_delegates() {
        let overlay = Overlay::new(TestWidget { label: "X".into() }, Anchor::Center, 10, 5);
        assert!(overlay.focusable());
    }
}
