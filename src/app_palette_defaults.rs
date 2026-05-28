//! Default palette values and theme constructors.

use txv_core::cell::{Attrs, Color, Style};

use super::*;

const fn fg(n: u8) -> Style {
    Style {
        fg: Color::Ansi(n),
        bg: Color::Reset,
        attrs: Attrs {
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        },
    }
}

const fn fgbg(f: u8, b: u8) -> Style {
    Style {
        fg: Color::Ansi(f),
        bg: Color::Ansi(b),
        attrs: Attrs {
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        },
    }
}

impl AppPalette {
    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        let mut p = Self::dark();
        *p.git_mut().modified_mut() = fg(4);
        *p.tree_mut().directory_mut() = fg(4);
        p
    }
}

impl Default for AppPalette {
    fn default() -> Self {
        Self::new(
            GitPalette::new(fg(2), fg(12), fg(1), fg(8), fg(5)),
            DiffPalette::new(fg(2), fg(1), fg(8)),
            EditorPalette::new(
                fg(8),
                fg(8),
                fgbg(0, 7),
                Style {
                    fg: Color::Reset,
                    bg: Color::Rgb(0x44, 0x44, 0x00),
                    attrs: Attrs::default(),
                },
                Style {
                    fg: Color::Reset,
                    bg: Color::Rgb(0x00, 0x44, 0x00),
                    attrs: Attrs::default(),
                },
                Style {
                    fg: Color::Reset,
                    bg: Color::Ansi(8),
                    attrs: Attrs {
                        bold: true,
                        ..Attrs::default()
                    },
                },
            ),
            DiagPalette::new(
                Style {
                    fg: Color::Ansi(1),
                    bg: Color::Reset,
                    attrs: Attrs {
                        underline: true,
                        ..Attrs::default()
                    },
                },
                Style {
                    fg: Color::Ansi(3),
                    bg: Color::Reset,
                    attrs: Attrs {
                        underline: true,
                        ..Attrs::default()
                    },
                },
                Style {
                    fg: Color::Ansi(6),
                    bg: Color::Reset,
                    attrs: Attrs {
                        underline: true,
                        ..Attrs::default()
                    },
                },
                Style {
                    fg: Color::Ansi(8),
                    bg: Color::Reset,
                    attrs: Attrs {
                        underline: true,
                        ..Attrs::default()
                    },
                },
            ),
            TreePalette::new(fg(14)),
            TodoPalette::new(fg(7), fg(8), fg(1)),
            MsgPalette::new(fg(9), fg(11), fg(7), fg(8)),
            BadgePalette::new(fg(2), fg(3), fg(1)),
        )
    }
}
