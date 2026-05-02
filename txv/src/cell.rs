// Cell, Color, Attrs, Style, Span — core types.

use std::env;

/// Terminal color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Color {
    /// Default terminal color.
    #[default]
    Reset,
    /// ANSI 16 colors (0–15).
    Ansi(u8),
    /// 256-color palette (0–255).
    Palette(u8),
    /// 24-bit RGB.
    Rgb(u8, u8, u8),
}

/// Text attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Attrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub dim: bool,
    pub strikethrough: bool,
}

/// Style = foreground + background + attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            attrs: Attrs::default(),
        }
    }
}

/// A single terminal cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
    /// Display width: 1 for normal, 2 for wide, 0 for continuation.
    pub width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
            width: 1,
        }
    }
}

/// A run of styled text.
#[derive(Clone, Debug)]
pub struct Span<'a> {
    pub text: &'a str,
    pub style: Style,
}

/// Detected terminal color capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Ansi,
    Palette,
    Rgb,
}

/// Detect terminal color capability from environment variables.
pub fn detect_color_mode() -> ColorMode {
    if let Ok(ct) = env::var("COLORTERM") {
        if ct == "truecolor" || ct == "24bit" {
            return ColorMode::Rgb;
        }
    }
    if let Ok(term) = env::var("TERM") {
        if term.contains("256color") {
            return ColorMode::Palette;
        }
    }
    ColorMode::Ansi
}

/// Convert RGB to nearest 256-color palette index.
pub fn rgb_to_palette(r: u8, g: u8, b: u8) -> u8 {
    // Check grayscale ramp (indices 232–255, 24 shades)
    let gray_match = nearest_gray(r, g, b);
    // Check 6×6×6 color cube (indices 16–231)
    let cube_match = nearest_cube(r, g, b);
    if gray_match.1 <= cube_match.1 {
        gray_match.0
    } else {
        cube_match.0
    }
}

/// Convert RGB to nearest ANSI 16-color index.
pub fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    let lum = (r as u32 + g as u32 + b as u32) / 3;
    let bright = lum > 127;
    let ri = if r > 127 { 1u8 } else { 0 };
    let gi = if g > 127 { 1u8 } else { 0 };
    let bi = if b > 127 { 1u8 } else { 0 };
    let base = bi << 2 | gi << 1 | ri;
    if bright {
        base + 8
    } else {
        base
    }
}

// --- helpers ---

fn nearest_cube(r: u8, g: u8, b: u8) -> (u8, u32) {
    let ri = cube_index(r);
    let gi = cube_index(g);
    let bi = cube_index(b);
    let idx = 16 + 36 * ri + 6 * gi + bi;
    let cr = cube_value(ri);
    let cg = cube_value(gi);
    let cb = cube_value(bi);
    let dist = color_dist(r, g, b, cr, cg, cb);
    (idx, dist)
}

fn cube_index(v: u8) -> u8 {
    match v {
        0..=47 => 0,
        48..=114 => 1,
        115..=154 => 2,
        155..=194 => 3,
        195..=234 => 4,
        _ => 5,
    }
}

fn cube_value(i: u8) -> u8 {
    if i == 0 {
        0
    } else {
        55 + 40 * i
    }
}

fn nearest_gray(r: u8, g: u8, b: u8) -> (u8, u32) {
    let avg = (r as u32 + g as u32 + b as u32) / 3;
    // Grayscale ramp: 232 + i, value = 8 + 10*i, for i in 0..24
    let i = if avg <= 3 {
        0u8
    } else if avg >= 243 {
        23
    } else {
        ((avg as u8).saturating_sub(8) + 5) / 10
    };
    let gv = 8 + 10 * i;
    let dist = color_dist(r, g, b, gv, gv, gv);
    (232 + i, dist)
}

fn color_dist(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    let dr = (r1 as i32) - (r2 as i32);
    let dg = (g1 as i32) - (g2 as i32);
    let db = (b1 as i32) - (b2 as i32);
    (dr * dr + dg * dg + db * db) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_is_reset() {
        let s = Style::default();
        assert_eq!(s.fg, Color::Reset);
        assert_eq!(s.bg, Color::Reset);
        assert_eq!(s.attrs, Attrs::default());
    }

    #[test]
    fn default_cell_is_space() {
        let c = Cell::default();
        assert_eq!(c.ch, ' ');
        assert_eq!(c.width, 1);
        assert_eq!(c.style, Style::default());
    }

    #[test]
    fn rgb_to_palette_black() {
        let idx = rgb_to_palette(0, 0, 0);
        // Should map to 16 (cube black) or 232 (gray ramp start=8)
        // cube(0,0,0) = index 16, value (0,0,0), dist=0
        assert_eq!(idx, 16);
    }

    #[test]
    fn rgb_to_palette_white() {
        let idx = rgb_to_palette(255, 255, 255);
        // cube(5,5,5) = 16+36*5+6*5+5 = 231, value (255,255,255), dist=0
        assert_eq!(idx, 231);
    }

    #[test]
    fn rgb_to_palette_red() {
        let idx = rgb_to_palette(255, 0, 0);
        // cube(5,0,0) = 16+180 = 196
        assert_eq!(idx, 196);
    }

    #[test]
    fn rgb_to_ansi_black() {
        assert_eq!(rgb_to_ansi(0, 0, 0), 0);
    }

    #[test]
    fn rgb_to_ansi_white() {
        assert_eq!(rgb_to_ansi(255, 255, 255), 15);
    }

    #[test]
    fn rgb_to_ansi_red() {
        let idx = rgb_to_ansi(255, 0, 0);
        // r>127, g<=127, b<=127, bright (lum=85 < 127 → not bright)
        assert_eq!(idx, 1); // dark red
    }

    #[test]
    fn color_mode_default() {
        // Without specific env vars, should be Ansi (or whatever env has)
        // Just test the function doesn't panic
        let _ = detect_color_mode();
    }
}
