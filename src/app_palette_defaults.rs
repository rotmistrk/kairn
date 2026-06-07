//! Default palette values and theme constructors.

use txv_core::cell::{Attrs, Color, Style};

use super::*;

fn fg(n: u8) -> Style {
    Style::new(Color::Ansi(n), Color::Reset)
}

fn fgbg(f: u8, b: u8) -> Style {
    Style::new(Color::Ansi(f), Color::Ansi(b))
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
            roots: default_roots_palette(),
        }
    }
}

fn default_editor_palette() -> EditorPalette {
    EditorPalette::new(
        fg(8),
        fg(8),
        fgbg(0, 7),
        Style::new(Color::Reset, Color::Rgb(0x44, 0x44, 0x00)),
        Style::new(Color::Reset, Color::Rgb(0x00, 0x44, 0x00)),
        Style::new(Color::Reset, Color::Ansi(8)).with_attrs(Attrs::default().bold()),
    )
}

fn default_diag_palette() -> DiagPalette {
    let underline = |color: u8| Style::new(Color::Ansi(color), Color::Reset).with_attrs(Attrs::default().underline());
    DiagPalette::new(underline(1), underline(3), underline(6), underline(8))
}

fn default_roots_palette() -> RootsPalette {
    // Excludes blue (active tab), gray/light-gray (inactive tab) to avoid confusion.
    RootsPalette::new(vec![
        Color::Ansi(2),  // green
        Color::Ansi(3),  // yellow/orange
        Color::Ansi(5),  // magenta
        Color::Ansi(6),  // cyan
        Color::Ansi(1),  // red
        Color::Ansi(13), // bright magenta
        Color::Ansi(14), // bright cyan
        Color::Ansi(11), // bright yellow
    ])
}
