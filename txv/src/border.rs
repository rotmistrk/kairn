// Border drawing — Pretty and CopyFriendly modes.

use crate::cell::Style;
use crate::layout::Rect;
use crate::surface::Surface;
use crate::text::display_width;

/// Border rendering mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderMode {
    /// Box-drawing characters (─│┌┐└┘).
    Pretty,
    /// Colored spaces (copy-friendly).
    CopyFriendly,
}

/// Border style configuration.
#[derive(Clone, Copy, Debug)]
pub struct BorderStyle {
    pub mode: BorderMode,
    pub active: Style,
    pub inactive: Style,
}

/// Draw a box border around a rect on a surface.
/// Returns the inner rect (area inside the border).
pub fn draw_border(
    surface: &mut Surface<'_>,
    rect: Rect,
    title: &str,
    style: &BorderStyle,
    focused: bool,
) -> Rect {
    let s = if focused {
        style.active
    } else {
        style.inactive
    };

    if rect.w < 2 || rect.h < 2 {
        return Rect {
            x: rect.x,
            y: rect.y,
            w: 0,
            h: 0,
        };
    }

    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        w: rect.w.saturating_sub(2),
        h: rect.h.saturating_sub(2),
    };

    match style.mode {
        BorderMode::Pretty => draw_pretty(surface, &rect, title, s),
        BorderMode::CopyFriendly => draw_copy_friendly(surface, &rect, title, s),
    }

    inner
}

fn draw_pretty(surface: &mut Surface<'_>, rect: &Rect, title: &str, style: Style) {
    let x = rect.x;
    let y = rect.y;
    let w = rect.w;
    let h = rect.h;
    let right = x + w - 1;
    let bottom = y + h - 1;

    // Corners
    surface.put(x, y, '┌', style);
    surface.put(right, y, '┐', style);
    surface.put(x, bottom, '└', style);
    surface.put(right, bottom, '┘', style);

    // Top and bottom horizontal lines
    surface.hline(x + 1, y, w.saturating_sub(2), '─', style);
    surface.hline(x + 1, bottom, w.saturating_sub(2), '─', style);

    // Left and right vertical lines
    surface.vline(x, y + 1, h.saturating_sub(2), '│', style);
    surface.vline(right, y + 1, h.saturating_sub(2), '│', style);

    // Title centered on top border
    render_title(surface, x, y, w, title, style);
}

fn draw_copy_friendly(surface: &mut Surface<'_>, rect: &Rect, title: &str, style: Style) {
    let x = rect.x;
    let y = rect.y;
    let w = rect.w;
    let h = rect.h;
    let right = x + w - 1;
    let bottom = y + h - 1;

    // Top row: spaces with border style
    surface.hline(x, y, w, ' ', style);
    // Bottom row
    surface.hline(x, bottom, w, ' ', style);
    // Left column
    surface.vline(x, y + 1, h.saturating_sub(2), ' ', style);
    // Right column
    surface.vline(right, y + 1, h.saturating_sub(2), ' ', style);

    // Title centered on top border
    render_title(surface, x, y, w, title, style);
}

fn render_title(surface: &mut Surface<'_>, x: u16, y: u16, w: u16, title: &str, style: Style) {
    if title.is_empty() || w < 5 {
        return;
    }
    let tw = display_width(title);
    let max_title = (w as usize).saturating_sub(4); // "─ " + " ─" = 4 chars
    if tw == 0 || max_title == 0 {
        return;
    }
    let display_title = if tw > max_title {
        crate::text::truncate(title, max_title)
    } else {
        title.to_string()
    };
    let dtw = display_width(&display_title);
    // Position: after "─ " (2 chars from left edge)
    let start = x + 2;
    surface.print(start, y, &display_title, style);
    // Add spaces around title
    if start > x + 1 {
        surface.put(start - 1, y, ' ', style);
    }
    let end = start + dtw as u16;
    if end < x + w - 1 {
        surface.put(end, y, ' ', style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{Color, ColorMode};
    use crate::screen::Screen;

    fn make_screen(w: u16, h: u16) -> Screen {
        Screen::with_color_mode(w, h, ColorMode::Rgb)
    }

    fn border_style() -> BorderStyle {
        BorderStyle {
            mode: BorderMode::Pretty,
            active: Style {
                fg: Color::Ansi(4),
                ..Style::default()
            },
            inactive: Style::default(),
        }
    }

    #[test]
    fn pretty_border_corners() {
        let mut screen = make_screen(20, 5);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 20,
            h: 5,
        };
        {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &border_style(), true);
        }
        assert_eq!(screen.cell(0, 0).ch, '┌');
        assert_eq!(screen.cell(19, 0).ch, '┐');
        assert_eq!(screen.cell(0, 4).ch, '└');
        assert_eq!(screen.cell(19, 4).ch, '┘');
    }

    #[test]
    fn pretty_border_lines() {
        let mut screen = make_screen(10, 5);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };
        {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &border_style(), true);
        }
        // Top horizontal
        assert_eq!(screen.cell(1, 0).ch, '─');
        assert_eq!(screen.cell(8, 0).ch, '─');
        // Left vertical
        assert_eq!(screen.cell(0, 1).ch, '│');
        assert_eq!(screen.cell(0, 3).ch, '│');
    }

    #[test]
    fn pretty_border_with_title() {
        let mut screen = make_screen(20, 5);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 20,
            h: 5,
        };
        {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "Title", &border_style(), true);
        }
        // Title should appear on top row
        let text = screen.to_text();
        assert!(text.contains("Title"));
    }

    #[test]
    fn inner_rect_calculation() {
        let mut screen = make_screen(20, 10);
        let rect = Rect {
            x: 2,
            y: 3,
            w: 15,
            h: 6,
        };
        let inner = {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &border_style(), true)
        };
        assert_eq!(inner.x, 3);
        assert_eq!(inner.y, 4);
        assert_eq!(inner.w, 13);
        assert_eq!(inner.h, 4);
    }

    #[test]
    fn copy_friendly_uses_spaces() {
        let mut screen = make_screen(10, 5);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };
        let cf_style = BorderStyle {
            mode: BorderMode::CopyFriendly,
            active: Style {
                bg: Color::Ansi(4),
                ..Style::default()
            },
            inactive: Style::default(),
        };
        {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &cf_style, true);
        }
        // All border cells should be spaces
        assert_eq!(screen.cell(0, 0).ch, ' ');
        assert_eq!(screen.cell(9, 0).ch, ' ');
        assert_eq!(screen.cell(0, 2).ch, ' ');
        // But they should have the border bg color
        assert_eq!(screen.cell(0, 0).style.bg, Color::Ansi(4));
    }

    #[test]
    fn tiny_rect_returns_zero_inner() {
        let mut screen = make_screen(10, 10);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        };
        let inner = {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &border_style(), true)
        };
        assert_eq!(inner.w, 0);
        assert_eq!(inner.h, 0);
    }

    #[test]
    fn focused_uses_active_style() {
        let mut screen = make_screen(10, 5);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };
        let bs = BorderStyle {
            mode: BorderMode::Pretty,
            active: Style {
                fg: Color::Rgb(100, 200, 150),
                ..Style::default()
            },
            inactive: Style {
                fg: Color::Ansi(8),
                ..Style::default()
            },
        };
        {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &bs, true);
        }
        assert_eq!(screen.cell(0, 0).style.fg, Color::Rgb(100, 200, 150));
    }

    #[test]
    fn unfocused_uses_inactive_style() {
        let mut screen = make_screen(10, 5);
        let rect = Rect {
            x: 0,
            y: 0,
            w: 10,
            h: 5,
        };
        let bs = BorderStyle {
            mode: BorderMode::Pretty,
            active: Style {
                fg: Color::Rgb(100, 200, 150),
                ..Style::default()
            },
            inactive: Style {
                fg: Color::Ansi(8),
                ..Style::default()
            },
        };
        {
            let mut s = screen.full_surface();
            draw_border(&mut s, rect, "", &bs, false);
        }
        assert_eq!(screen.cell(0, 0).style.fg, Color::Ansi(8));
    }
}
