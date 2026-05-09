//! Color mode detection and downgrade (RGB → 256 → 16).

use txv_core::cell::Color;

/// Terminal color capability level.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorMode {
    TrueColor,
    Palette256,
    Ansi16,
}

/// Detect terminal color capability from environment.
pub fn detect_color_mode() -> ColorMode {
    if let Ok(val) = std::env::var("COLORTERM") {
        if val == "truecolor" || val == "24bit" {
            return ColorMode::TrueColor;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("256color") {
            return ColorMode::Palette256;
        }
    }
    ColorMode::Ansi16
}

/// Downgrade a Color to fit the given ColorMode.
pub fn downgrade(color: Color, mode: ColorMode) -> Color {
    match (color, mode) {
        (Color::Reset, _) => Color::Reset,
        (Color::Ansi(n), _) => Color::Ansi(n),
        (Color::Palette(n), ColorMode::TrueColor | ColorMode::Palette256) => {
            Color::Palette(n)
        }
        (Color::Palette(n), ColorMode::Ansi16) => {
            Color::Ansi(palette_to_ansi(n))
        }
        (Color::Rgb(r, g, b), ColorMode::TrueColor) => Color::Rgb(r, g, b),
        (Color::Rgb(r, g, b), ColorMode::Palette256) => {
            Color::Palette(rgb_to_palette(r, g, b))
        }
        (Color::Rgb(r, g, b), ColorMode::Ansi16) => {
            Color::Ansi(rgb_to_ansi(r, g, b))
        }
    }
}

/// Convert 256-palette index to nearest ANSI 16 color.
fn palette_to_ansi(n: u8) -> u8 {
    if n < 16 {
        return n;
    }
    if n >= 232 {
        // Grayscale ramp: 232..=255 → map to 0(black) or 7(white) or 8/15
        let level = (n - 232) * 10 + 8;
        return if level < 64 {
            0
        } else if level < 128 {
            8
        } else if level < 192 {
            7
        } else {
            15
        };
    }
    // 6x6x6 cube: index 16..=231
    let idx = n - 16;
    let b = idx % 6;
    let g = (idx / 6) % 6;
    let r = idx / 36;
    cube_to_ansi(r, g, b)
}

/// Map 6x6x6 cube coordinates to ANSI 16.
fn cube_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    let bright = r >= 3 || g >= 3 || b >= 3;
    let base = ((b >= 3) as u8) << 2
        | ((g >= 3) as u8) << 1
        | (r >= 3) as u8;
    // If all are low but nonzero, use dark variant
    if !bright && (r > 0 || g > 0 || b > 0) {
        let base2 = ((b >= 2) as u8) << 2
            | ((g >= 2) as u8) << 1
            | (r >= 2) as u8;
        return base2;
    }
    if bright { base + 8 } else { base }
}

/// Convert RGB to nearest 256-palette index.
fn rgb_to_palette(r: u8, g: u8, b: u8) -> u8 {
    // Check if it's close to a grayscale
    let gray_candidate = if r == g && g == b {
        Some(r)
    } else {
        None
    };

    if let Some(level) = gray_candidate {
        if level < 8 {
            return 16; // black-ish in cube
        }
        if level > 248 {
            return 231; // white-ish in cube
        }
        return 232 + ((level as u16 - 8) * 24 / 240) as u8;
    }

    // Map to 6x6x6 cube
    let ri = color_to_6(r);
    let gi = color_to_6(g);
    let bi = color_to_6(b);
    16 + 36 * ri + 6 * gi + bi
}

fn color_to_6(v: u8) -> u8 {
    match v {
        0..=47 => 0,
        48..=114 => 1,
        115..=154 => 2,
        155..=194 => 3,
        195..=234 => 4,
        _ => 5,
    }
}

/// Convert RGB directly to nearest ANSI 16 color.
fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    palette_to_ansi(rgb_to_palette(r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_unchanged() {
        assert_eq!(downgrade(Color::Reset, ColorMode::Ansi16), Color::Reset);
    }

    #[test]
    fn ansi_unchanged() {
        assert_eq!(
            downgrade(Color::Ansi(5), ColorMode::Ansi16),
            Color::Ansi(5)
        );
    }

    #[test]
    fn rgb_to_palette_black() {
        assert_eq!(rgb_to_palette(0, 0, 0), 16);
    }

    #[test]
    fn rgb_to_palette_white() {
        assert_eq!(rgb_to_palette(255, 255, 255), 231);
    }

    #[test]
    fn rgb_to_palette_red() {
        // Pure red → cube index (5,0,0) → 16 + 36*5 = 196
        assert_eq!(rgb_to_palette(255, 0, 0), 196);
    }

    #[test]
    fn downgrade_rgb_truecolor_passthrough() {
        assert_eq!(
            downgrade(Color::Rgb(100, 200, 50), ColorMode::TrueColor),
            Color::Rgb(100, 200, 50)
        );
    }

    #[test]
    fn downgrade_rgb_to_256() {
        let result = downgrade(Color::Rgb(255, 0, 0), ColorMode::Palette256);
        assert_eq!(result, Color::Palette(196));
    }

    #[test]
    fn downgrade_rgb_to_ansi16() {
        let result = downgrade(Color::Rgb(255, 0, 0), ColorMode::Ansi16);
        // Red → palette 196 → cube (5,0,0) → ansi bright red (9)
        assert_eq!(result, Color::Ansi(9));
    }

    #[test]
    fn palette_low_unchanged_in_ansi16() {
        assert_eq!(
            downgrade(Color::Palette(3), ColorMode::Ansi16),
            Color::Ansi(3)
        );
    }
}
