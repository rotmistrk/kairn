//! Editor sub-palette.

use txv_core::cell::Style;

#[derive(Clone, Debug)]
pub struct EditorPalette {
    gutter: Style,
    list_chars: Style,
    cursor: Style,
    highlight_match: Style,
    highlight_other: Style,
    matchparen: Style,
}

impl EditorPalette {
    pub fn new(
        gutter: Style,
        list_chars: Style,
        cursor: Style,
        highlight_match: Style,
        highlight_other: Style,
        matchparen: Style,
    ) -> Self {
        Self {
            gutter,
            list_chars,
            cursor,
            highlight_match,
            highlight_other,
            matchparen,
        }
    }

    pub fn gutter(&self) -> Style {
        self.gutter
    }
    pub fn list_chars(&self) -> Style {
        self.list_chars
    }
    pub fn cursor(&self) -> Style {
        self.cursor
    }
    pub fn highlight_match(&self) -> Style {
        self.highlight_match
    }
    pub fn highlight_other(&self) -> Style {
        self.highlight_other
    }
    pub fn matchparen(&self) -> Style {
        self.matchparen
    }

    pub fn gutter_mut(&mut self) -> &mut Style {
        &mut self.gutter
    }
    pub fn list_chars_mut(&mut self) -> &mut Style {
        &mut self.list_chars
    }
}
