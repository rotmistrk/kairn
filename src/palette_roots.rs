//! Root badge sub-palette — colors for multi-root workspace badges.

use txv_core::cell::Color;

#[derive(Clone, Debug)]
pub struct RootsPalette {
    colors: Vec<Color>,
}

impl RootsPalette {
    pub fn new(colors: Vec<Color>) -> Self {
        Self { colors }
    }

    pub fn color_at(&self, index: usize) -> Color {
        self.colors[index % self.colors.len()]
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}
