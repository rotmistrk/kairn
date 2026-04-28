pub mod commit_tree;
pub mod file_tree;
pub mod interactive;
pub mod main_view;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Tree,
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

/// Adjust scroll offset so `cursor` stays visible within `height` rows.
pub fn adjust_scroll(cursor: usize, current: usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    if cursor < current {
        cursor
    } else if cursor >= current + height {
        cursor - height + 1
    } else {
        current
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
    SwitchMode,
    SendToKiro(String),
    PreviewCommit(String),
    ExpandLine,
    Yank(String),
    FocusRight,
    FocusLeft,
    PushOutput(crate::buffer::OutputBuffer),
    Quit,
}
