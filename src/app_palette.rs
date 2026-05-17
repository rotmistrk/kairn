//! Application-level palette — extends txv-core Palette with domain-specific roles.

use std::sync::OnceLock;

use txv_core::cell::Color;
use txv_core::palette::{Palette, PaletteStyle};

static APP_PALETTE: OnceLock<std::sync::RwLock<AppPalette>> = OnceLock::new();

/// Get the active app palette.
pub fn app_palette() -> AppPalette {
    match APP_PALETTE.get() {
        Some(lock) => lock.read().map(|p| p.clone()).unwrap_or_default(),
        None => AppPalette::default(),
    }
}

/// Set the active app palette.
pub fn set_app_palette(p: &AppPalette) {
    match APP_PALETTE.get() {
        Some(lock) => {
            if let Ok(mut w) = lock.write() {
                *w = p.clone();
            }
        }
        None => {
            let _ = APP_PALETTE.set(std::sync::RwLock::new(p.clone()));
        }
    }
}

/// kairn-specific palette extending the framework palette.
#[derive(Clone, Debug)]
pub struct AppPalette {
    pub base: Palette,
    pub git: GitPalette,
    pub diff: DiffPalette,
    pub editor: EditorPalette,
    pub diag: DiagPalette,
    pub tree: TreePalette,
    pub todo: TodoPalette,
    pub msg: MsgPalette,
    pub badge: BadgePalette,
}

#[derive(Clone, Debug)]
pub struct GitPalette {
    pub added: PaletteStyle,
    pub modified: PaletteStyle,
    pub untracked: PaletteStyle,
    pub ignored: PaletteStyle,
    pub conflict: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct DiffPalette {
    pub added: PaletteStyle,
    pub deleted: PaletteStyle,
    pub fold: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct EditorPalette {
    pub gutter: PaletteStyle,
    pub list_chars: PaletteStyle,
    pub cursor: PaletteStyle,
    pub highlight_match: PaletteStyle,
    pub highlight_other: PaletteStyle,
    pub matchparen: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct DiagPalette {
    pub error: PaletteStyle,
    pub warning: PaletteStyle,
    pub info: PaletteStyle,
    pub hint: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct TreePalette {
    pub directory: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct TodoPalette {
    pub normal: PaletteStyle,
    pub done: PaletteStyle,
    pub important: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct MsgPalette {
    pub error: PaletteStyle,
    pub warning: PaletteStyle,
    pub info: PaletteStyle,
    pub debug: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct BadgePalette {
    pub busy: PaletteStyle,
    pub idle: PaletteStyle,
    pub exited: PaletteStyle,
}

const fn ansi(n: u8) -> Color {
    Color::Ansi(n)
}

impl AppPalette {
    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        let mut p = Self::dark();
        p.base = Palette::light();
        p.git.modified = PaletteStyle::fg(ansi(4));
        p.tree.directory = PaletteStyle::fg(ansi(4));
        p
    }
}

impl Default for AppPalette {
    fn default() -> Self {
        let mut base = Palette::dark();
        // Darker backgrounds for interactive elements
        base.interactive.cursor_focused = PaletteStyle {
            fg: None,
            bg: Some(Color::Rgb(0x00, 0x00, 0x66)),
            attrs: None,
        };
        base.interactive.visual_selection = PaletteStyle {
            fg: None,
            bg: Some(Color::Rgb(0x00, 0x44, 0x44)),
            attrs: None,
        };
        base.interactive.edit_overlay = PaletteStyle {
            fg: None,
            bg: Some(Color::Rgb(0x00, 0x44, 0x00)),
            attrs: None,
        };
        Self {
            base,
            git: GitPalette {
                added: PaletteStyle::fg(ansi(2)),
                modified: PaletteStyle::fg(ansi(12)),
                untracked: PaletteStyle::fg(ansi(1)),
                ignored: PaletteStyle::fg(ansi(8)),
                conflict: PaletteStyle::fg(ansi(5)),
            },
            diff: DiffPalette {
                added: PaletteStyle::fg(ansi(2)),
                deleted: PaletteStyle::fg(ansi(1)),
                fold: PaletteStyle::fg(ansi(8)),
            },
            editor: EditorPalette {
                gutter: PaletteStyle::fg(ansi(8)),
                list_chars: PaletteStyle::fg(ansi(8)),
                cursor: PaletteStyle {
                    attrs: Some(txv_core::cell::Attrs {
                        reverse: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                highlight_match: PaletteStyle {
                    fg: None,
                    bg: Some(Color::Rgb(0x44, 0x44, 0x00)),
                    attrs: None,
                },
                highlight_other: PaletteStyle {
                    fg: None,
                    bg: Some(Color::Rgb(0x00, 0x44, 0x00)),
                    attrs: None,
                },
                matchparen: PaletteStyle {
                    fg: None,
                    bg: Some(ansi(8)),
                    attrs: Some(txv_core::cell::Attrs {
                        bold: true,
                        ..Default::default()
                    }),
                },
            },
            diag: DiagPalette {
                error: PaletteStyle {
                    fg: Some(ansi(1)),
                    attrs: Some(txv_core::cell::Attrs {
                        underline: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                warning: PaletteStyle {
                    fg: Some(ansi(3)),
                    attrs: Some(txv_core::cell::Attrs {
                        underline: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                info: PaletteStyle {
                    fg: Some(ansi(6)),
                    attrs: Some(txv_core::cell::Attrs {
                        underline: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                hint: PaletteStyle {
                    fg: Some(ansi(8)),
                    attrs: Some(txv_core::cell::Attrs {
                        underline: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
            tree: TreePalette {
                directory: PaletteStyle::fg(ansi(14)),
            },
            todo: TodoPalette {
                normal: PaletteStyle::fg(ansi(7)),
                done: PaletteStyle::fg(ansi(8)),
                important: PaletteStyle::fg(ansi(1)),
            },
            msg: MsgPalette {
                error: PaletteStyle::fg(ansi(9)),
                warning: PaletteStyle::fg(ansi(11)),
                info: PaletteStyle::fg(ansi(7)),
                debug: PaletteStyle::fg(ansi(8)),
            },
            badge: BadgePalette {
                busy: PaletteStyle::fg(ansi(2)),
                idle: PaletteStyle::fg(ansi(3)),
                exited: PaletteStyle::fg(ansi(1)),
            },
        }
    }
}
