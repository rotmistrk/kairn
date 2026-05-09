//! Status bar — left/right aligned styled spans.

use txv::cell::Style;
use txv::layout::Rect;
use txv::surface::Surface;
use txv::text::display_width;

use crate::view::{DrawContext, Event, HandleResult, View};

/// A styled text span for the status bar.
pub struct StatusSpan {
    /// Display text.
    pub text: String,
    /// Style for this span.
    pub style: Style,
}

/// A single-row bar with left-aligned and right-aligned spans.
pub struct StatusBar {
    left: Vec<StatusSpan>,
    right: Vec<StatusSpan>,
    bg: Style,
    bounds: Rect,
}

impl StatusBar {
    /// Create an empty status bar.
    pub fn new() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
            bg: Style::default(),
            bounds: Rect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        }
    }

    /// Set the left-aligned spans.
    pub fn set_left(&mut self, spans: Vec<StatusSpan>) {
        self.left = spans;
    }

    /// Set the right-aligned spans.
    pub fn set_right(&mut self, spans: Vec<StatusSpan>) {
        self.right = spans;
    }

    /// Set the background fill style.
    pub fn set_bg(&mut self, style: Style) {
        self.bg = style;
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for StatusBar {
    fn draw(&self, surface: &mut Surface<'_>, _ctx: &DrawContext) {
        let w = surface.width();
        // Fill background
        surface.hline(0, 0, w, ' ', self.bg);

        // Render left spans
        let mut col: u16 = 0;
        for span in &self.left {
            if col >= w {
                break;
            }
            surface.print(col, 0, &span.text, span.style);
            col += display_width(&span.text) as u16;
        }

        // Render right spans (right-aligned)
        let right_width: usize = self.right.iter().map(|s| display_width(&s.text)).sum();
        let start = (w as usize).saturating_sub(right_width) as u16;
        let mut rcol = start;
        for span in &self.right {
            if rcol >= w {
                break;
            }
            surface.print(rcol, 0, &span.text, span.style);
            rcol += display_width(&span.text) as u16;
        }
    }

    fn handle(&mut self, _event: &Event) -> HandleResult {
        HandleResult::Ignored
    }

    fn focusable(&self) -> bool {
        false
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use txv::cell::{Color, ColorMode};
    use txv::screen::Screen;

    fn render_bar(bar: &StatusBar, width: u16) -> String {
        let mut screen = Screen::with_color_mode(width, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            bar.draw(
                &mut s,
                &DrawContext {
                    app_focused: true,
                    tick: 0,
                },
            );
        }
        let text = screen.to_text();
        text.trim_end_matches('\n').to_string()
    }

    #[test]
    fn empty_bar_fills_bg() {
        let bar = StatusBar::new();
        let text = render_bar(&bar, 10);
        assert_eq!(text, "          ");
    }

    #[test]
    fn left_spans_render() {
        let mut bar = StatusBar::new();
        bar.set_left(vec![
            StatusSpan {
                text: "AB".into(),
                style: Style::default(),
            },
            StatusSpan {
                text: "CD".into(),
                style: Style::default(),
            },
        ]);
        let text = render_bar(&bar, 10);
        assert!(text.starts_with("ABCD"));
    }

    #[test]
    fn right_spans_render() {
        let mut bar = StatusBar::new();
        bar.set_right(vec![StatusSpan {
            text: "XY".into(),
            style: Style::default(),
        }]);
        let text = render_bar(&bar, 10);
        assert!(text.ends_with("XY"));
        assert_eq!(&text[8..], "XY");
    }

    #[test]
    fn left_and_right_together() {
        let mut bar = StatusBar::new();
        bar.set_left(vec![StatusSpan {
            text: "L".into(),
            style: Style::default(),
        }]);
        bar.set_right(vec![StatusSpan {
            text: "R".into(),
            style: Style::default(),
        }]);
        let text = render_bar(&bar, 10);
        assert!(text.starts_with('L'));
        assert!(text.ends_with('R'));
    }

    #[test]
    fn overflow_left_truncates() {
        let mut bar = StatusBar::new();
        bar.set_left(vec![StatusSpan {
            text: "ABCDEFGHIJKLMNO".into(),
            style: Style::default(),
        }]);
        let text = render_bar(&bar, 5);
        assert_eq!(text.len(), 5);
        assert_eq!(&text, "ABCDE");
    }

    #[test]
    fn right_spans_use_style() {
        let mut bar = StatusBar::new();
        let style = Style {
            fg: Color::Ansi(1),
            ..Style::default()
        };
        bar.set_right(vec![StatusSpan {
            text: "R".into(),
            style,
        }]);
        let mut screen = Screen::with_color_mode(10, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            bar.draw(
                &mut s,
                &DrawContext {
                    app_focused: true,
                    tick: 0,
                },
            );
        }
        assert_eq!(screen.cell(9, 0).style.fg, Color::Ansi(1));
    }

    #[test]
    fn not_focusable() {
        let bar = StatusBar::new();
        assert!(!bar.focusable());
    }

    #[test]
    fn handle_key_ignored() {
        let mut bar = StatusBar::new();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(matches!(
            bar.handle(&Event::Key(key)),
            HandleResult::Ignored
        ));
    }
}
