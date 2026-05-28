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
        Self {
            git: GitPalette::new(fg(2), fg(12), fg(1), fg(8), fg(5)),
            diff: DiffPalette::new(fg(2), fg(1), fg(8)),
            editor: default_editor_palette(),
            diag: default_diag_palette(),
            tree: TreePalette::new(fg(14)),
            todo: TodoPalette::new(fg(7), fg(8), fg(1)),
            msg: MsgPalette::new(fg(9), fg(11), fg(7), fg(8)),
            badge: BadgePalette::new(fg(2), fg(3), fg(1)),
        }
    }
}

fn default_editor_palette() -> EditorPalette {
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
    )
}

fn default_diag_palette() -> DiagPalette {
    let underline = |color: u8| Style {
        fg: Color::Ansi(color),
        bg: Color::Reset,
        attrs: Attrs {
            underline: true,
            ..Attrs::default()
        },
    };
    DiagPalette::new(underline(1), underline(3), underline(6), underline(8))
}
