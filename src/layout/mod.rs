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
    /// Interactive panel (Kiro/Shell) size — width in Wide, height otherwise
    pub interactive_size: u16,
}

impl Default for PanelSizes {
    fn default() -> Self {
        Self {
            tree_width: 30,
            interactive_size: 40,
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

    pub fn resize_interactive(&mut self, delta: i16) {
        let new_val = (self.interactive_size as i16).saturating_add(delta);
        self.interactive_size = new_val.clamp(5, 200) as u16;
    }
}
