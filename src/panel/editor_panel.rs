//! Editor panel (triptych): tree + editor view + control panel.

use crossterm::event::KeyEvent;
use txv::surface::Surface;
use txv_widgets::{EventResult, TabBar, TabEntry, Widget, WidgetAction};

use super::control_panel::ControlPanel;
use super::editor_view::EditorView;
use super::tree_panel::TreePanel;
use super::TriptychFocus;

/// The main editing area: tree + editor + control, with file tabs.
pub struct EditorPanel {
    pub tree: TreePanel,
    pub editor: EditorView,
    pub control: ControlPanel,
    pub tree_visible: bool,
    pub control_visible: bool,
    pub focus: TriptychFocus,
    pub file_tabs: TabBar,
    open_files: Vec<String>,
}

impl EditorPanel {
    /// Create a new editor panel.
    pub fn new(tree: TreePanel) -> Self {
        Self {
            tree,
            editor: EditorView::new(),
            control: ControlPanel::new(),
            tree_visible: true,
            control_visible: false,
            focus: TriptychFocus::Editor,
            file_tabs: TabBar::new(),
            open_files: Vec::new(),
        }
    }

    /// Open a file in the editor.
    pub fn open_file(&mut self, path: &str) -> anyhow::Result<()> {
        // Check if already open.
        if let Some(idx) = self.open_files.iter().position(|p| p == path) {
            self.file_tabs.set_active(idx);
            // TODO: switch to that buffer
            return Ok(());
        }

        self.editor.open_file(path)?;
        self.open_files.push(path.to_string());
        self.file_tabs.add(TabEntry {
            title: short_name(path),
            modified: false,
        });
        self.file_tabs.set_active(self.open_files.len() - 1);
        Ok(())
    }

    /// Toggle tree visibility.
    pub fn toggle_tree(&mut self) {
        self.tree_visible = !self.tree_visible;
        if !self.tree_visible && self.focus == TriptychFocus::Tree {
            self.focus = TriptychFocus::Editor;
        }
    }

    /// Toggle control panel visibility.
    pub fn toggle_control(&mut self) {
        self.control_visible = !self.control_visible;
        if !self.control_visible && self.focus == TriptychFocus::Control {
            self.focus = TriptychFocus::Editor;
        }
    }

    /// Cycle focus within the triptych.
    pub fn cycle_focus_right(&mut self) {
        self.focus = match self.focus {
            TriptychFocus::Tree => TriptychFocus::Editor,
            TriptychFocus::Editor => {
                if self.control_visible {
                    TriptychFocus::Control
                } else if self.tree_visible {
                    TriptychFocus::Tree
                } else {
                    TriptychFocus::Editor
                }
            }
            TriptychFocus::Control => {
                if self.tree_visible {
                    TriptychFocus::Tree
                } else {
                    TriptychFocus::Editor
                }
            }
        };
    }

    /// Cycle focus left within the triptych.
    pub fn cycle_focus_left(&mut self) {
        self.focus = match self.focus {
            TriptychFocus::Tree => {
                if self.control_visible {
                    TriptychFocus::Control
                } else {
                    TriptychFocus::Editor
                }
            }
            TriptychFocus::Editor => {
                if self.tree_visible {
                    TriptychFocus::Tree
                } else if self.control_visible {
                    TriptychFocus::Control
                } else {
                    TriptychFocus::Editor
                }
            }
            TriptychFocus::Control => TriptychFocus::Editor,
        };
    }
}

impl Widget for EditorPanel {
    fn render(&self, _surface: &mut Surface<'_>, _focused: bool) {
        // Rendering is handled by App which renders each sub-panel
        // into its own computed rect. This is a no-op.
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        match self.focus {
            TriptychFocus::Tree => {
                let result = self.tree.handle_key(key);
                if let EventResult::Action(WidgetAction::Selected(ref path)) = result {
                    EventResult::Action(WidgetAction::Selected(path.clone()))
                } else {
                    result
                }
            }
            TriptychFocus::Editor => self.editor.handle_key(key),
            TriptychFocus::Control => self.control.handle_key(key),
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}

/// Extract short filename from path.
fn short_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}
