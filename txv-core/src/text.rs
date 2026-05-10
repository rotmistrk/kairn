//! Text measurement utilities — display width, visual positions.

/// Display width of a character (1 for normal, 2 for wide/CJK).
pub fn display_char_width(ch: char) -> u16 {
    let cp = ch as u32;
    if (0x1100..=0x115F).contains(&cp)
        || (0x2E80..=0x303E).contains(&cp)
        || (0x3041..=0x33BF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFE30..=0xFE6F).contains(&cp)
        || (0xFF01..=0xFF60).contains(&cp)
        || (0xFFE0..=0xFFE6).contains(&cp)
        || (0x20000..=0x2FFFD).contains(&cp)
        || (0x30000..=0x3FFFD).contains(&cp)
        || (0x2600..=0x27BF).contains(&cp)  // Misc symbols (✅, etc.)
        || (0x1F300..=0x1F9FF).contains(&cp)
    // Emoji
    {
        2
    } else {
        1
    }
}

/// Iterate characters with their visual column positions.
/// Handles wide chars (2 cells) and tabs (tab_width cells).
/// Apps use this instead of manual col tracking.
///
/// Returns: Vec of (visual_col, char, char_display_width).
///
/// # Example
///
/// ```
/// use txv_core::text::{visual_positions, display_width};
/// # use txv_core::prelude::*;
/// # let mut surface = Surface::new(40, 1);
/// # let x = 0u16;
/// # let y = 0u16;
/// # let style = Style::default();
/// for (col, ch, _width) in visual_positions("hello ✅ world", 4) {
///     surface.put(x + col, y, ch, style);
/// }
/// // Padding starts at x + display_width("hello ✅ world", 4)
/// ```
pub fn visual_positions(text: &str, tab_width: usize) -> Vec<(u16, char, u16)> {
    let mut col: u16 = 0;
    let mut result = Vec::new();
    for ch in text.chars() {
        let w = if ch == '\t' {
            tab_width as u16
        } else {
            display_char_width(ch)
        };
        result.push((col, ch, w));
        col += w;
    }
    result
}

/// Total display width of a string (accounts for wide chars and tabs).
pub fn display_width(text: &str, tab_width: usize) -> u16 {
    let mut col: u16 = 0;
    for ch in text.chars() {
        col += if ch == '\t' {
            tab_width as u16
        } else {
            display_char_width(ch)
        };
    }
    col
}
