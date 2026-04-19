pub mod file_tree;
pub mod interactive;
pub mod main_view;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    Tree,
    #[default]
    Main,
    Interactive,
}

impl FocusedPanel {
    pub fn next(self) -> Self {
        match self {
            Self::Tree => Self::Main,
            Self::Main => Self::Interactive,
            Self::Interactive => Self::Tree,
        }
    }
}

/// Trait for renderable, focusable panels.
pub trait Panel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction>;
}

/// Actions a panel can request from the app.
#[derive(Debug)]
pub enum PanelAction {
    None,
    OpenFile(String),
    PreviewFile(String),
    PushOutput(crate::buffer::OutputBuffer),
    Quit,
}
