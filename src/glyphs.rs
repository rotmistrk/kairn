//! Glyph sets for chrome rendering.
//! Default: Nerd Font. Future: configurable via .kairnrc "glyphs": "ascii".

pub struct Glyphs {
    pub(crate) tab_left: &'static str,
    pub(crate) tab_right: &'static str,
    pub(crate) dropdown_arrow: &'static str,
    pub(crate) badge_left: &'static str,
    pub(crate) badge_right: &'static str,
    pub(crate) check: &'static str,
    pub(crate) cross: &'static str,
}

/// Nerd Font / Powerline glyphs (default).
pub const NERD: Glyphs = Glyphs {
    tab_left: "\u{E0B6}",  //
    tab_right: "\u{E0B4}", //
    dropdown_arrow: "▾",
    badge_left: "\u{E0B6}",
    badge_right: "\u{E0B4}",
    check: "✓",
    cross: "✗",
};

/// ASCII-safe fallback.
pub const ASCII: Glyphs = Glyphs {
    tab_left: "[",
    tab_right: "]",
    dropdown_arrow: "v",
    badge_left: "(",
    badge_right: ")",
    check: "[x]",
    cross: "[ ]",
};

/// Active glyph set. TODO: make configurable via .kairnrc.
pub fn glyphs() -> &'static Glyphs {
    &NERD
}
