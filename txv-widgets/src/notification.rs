//! Flash message with auto-dismiss countdown.

use txv::cell::Style;
use txv::surface::Surface;

/// A notification that counts down and expires.
pub struct Notification {
    /// The message to display.
    pub message: String,
    /// Style for the notification.
    pub style: Style,
    remaining_ticks: u32,
}

impl Notification {
    /// Create a new notification with a tick-based countdown.
    pub fn new(message: &str, style: Style, ticks: u32) -> Self {
        Self {
            message: message.to_string(),
            style,
            remaining_ticks: ticks,
        }
    }

    /// Decrement the countdown. Returns `false` when expired.
    pub fn tick(&mut self) -> bool {
        self.remaining_ticks = self.remaining_ticks.saturating_sub(1);
        self.remaining_ticks > 0
    }

    /// Whether the notification has expired.
    pub fn expired(&self) -> bool {
        self.remaining_ticks == 0
    }

    /// Render the notification into a surface (single row).
    pub fn render(&self, surface: &mut Surface<'_>) {
        let w = surface.width();
        surface.hline(0, 0, w, ' ', self.style);
        surface.print(0, 0, &self.message, self.style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    #[test]
    fn new_not_expired() {
        let n = Notification::new("hello", Style::default(), 5);
        assert!(!n.expired());
    }

    #[test]
    fn tick_countdown() {
        let mut n = Notification::new("msg", Style::default(), 3);
        assert!(n.tick()); // 2 remaining
        assert!(n.tick()); // 1 remaining
        assert!(!n.tick()); // 0 remaining, expired
        assert!(n.expired());
    }

    #[test]
    fn tick_zero_immediately_expired() {
        let n = Notification::new("msg", Style::default(), 0);
        assert!(n.expired());
    }

    #[test]
    fn tick_saturates_at_zero() {
        let mut n = Notification::new("msg", Style::default(), 1);
        assert!(!n.tick());
        assert!(!n.tick()); // stays at 0
        assert!(n.expired());
    }

    #[test]
    fn render_shows_message() {
        let n = Notification::new("Alert!", Style::default(), 5);
        let mut screen = Screen::with_color_mode(20, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            n.render(&mut s);
        }
        let text = screen.to_text();
        assert!(text.contains("Alert!"));
    }

    #[test]
    fn render_uses_style() {
        let style = Style {
            fg: txv::cell::Color::Ansi(1),
            ..Style::default()
        };
        let n = Notification::new("X", style, 5);
        let mut screen = Screen::with_color_mode(10, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            n.render(&mut s);
        }
        assert_eq!(screen.cell(0, 0).style.fg, txv::cell::Color::Ansi(1));
    }
}
