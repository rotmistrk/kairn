//! Glyph sets for chrome rendering.
//! Default: Nerd Font. Future: configurable via .kairnrc "glyphs": "ascii".

pub struct Glyphs {
    pub tab_left: &'static str,
    pub tab_right: &'static str,
    pub dropdown_arrow: &'static str,
    pub badge_left: &'static str,
    pub badge_right: &'static str,
}

/// Nerd Font / Powerline glyphs (default).
pub const NERD: Glyphs = Glyphs {
    tab_left: "\u{E0B6}",  //
    tab_right: "\u{E0B4}", //
    dropdown_arrow: "▾",
    badge_left: "\u{E0B6}",
    badge_right: "\u{E0B4}",
};

/// ASCII-safe fallback.
pub const ASCII: Glyphs = Glyphs {
    tab_left: "[",
    tab_right: "]",
    dropdown_arrow: "v",
    badge_left: "(",
    badge_right: ")",
};

/// Active glyph set. TODO: make configurable via .kairnrc.
pub fn glyphs() -> &'static Glyphs {
    &NERD
}
