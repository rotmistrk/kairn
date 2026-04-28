mod constraints;

pub use constraints::LayoutConstraints;

use serde::{Deserialize, Serialize};

/// The three layout modes, cycled with Ctrl-L.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayoutMode {
    /// Wide: Tree | Main | Kiro/Shell (three columns)
    #[default]
    Wide,
    /// Tall-right: Tree | (Main top / Kiro bottom)
    TallRight,
    /// Tall-bottom: (Tree top / Main right) over Kiro full-width bottom
    TallBottom,
}

impl LayoutMode {
    pub fn next(self) -> Self {
        match self {
            Self::Wide => Self::TallRight,
            Self::TallRight => Self::TallBottom,
            Self::TallBottom => Self::Wide,
        }
    }
}

/// Panel sizing state, adjustable via hotkeys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSizes {
    /// File tree width in columns (0 = hidden)
    pub tree_width: u16,
    /// Interactive panel width in columns (Wide layout)
    #[serde(alias = "interactive_size")]
    #[serde(default = "default_interactive_width")]
    pub interactive_width: u16,
    /// Interactive panel height in rows (stacked layouts)
    #[serde(default = "default_interactive_height")]
    pub interactive_height: u16,
}

fn default_interactive_width() -> u16 {
    40
}

fn default_interactive_height() -> u16 {
    15
}

impl Default for PanelSizes {
    fn default() -> Self {
        Self {
            tree_width: 30,
            interactive_width: default_interactive_width(),
            interactive_height: default_interactive_height(),
        }
    }
}

impl PanelSizes {
    pub fn toggle_tree(&mut self) {
        if self.tree_width == 0 {
            self.tree_width = 30;
        } else {
            self.tree_width = 0;
        }
    }

    pub fn resize_tree(&mut self, delta: i16) {
        let new_val = (self.tree_width as i16).saturating_add(delta);
        self.tree_width = new_val.clamp(0, 80) as u16;
    }

    pub fn resize_interactive_width(&mut self, delta: i16) {
        let new_val = (self.interactive_width as i16).saturating_add(delta);
        self.interactive_width = new_val.clamp(5, 200) as u16;
    }

    pub fn resize_interactive_height(&mut self, delta: i16) {
        let new_val = (self.interactive_height as i16).saturating_add(delta);
        self.interactive_height = new_val.clamp(3, 200) as u16;
    }
}
