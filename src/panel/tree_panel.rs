//! Tree panel: wraps txv-widgets TreeView with FileTreeData.

use std::path::{Path, PathBuf};

use crossterm::event::KeyEvent;
use txv::surface::Surface;
use txv_widgets::{EventResult, FileTreeData, TreeView, Widget};

/// Which tree mode is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeMode {
    #[default]
    Files,
    Git,
    Symbols,
    Packages,
}

/// File tree panel wrapping a `TreeView<FileTreeData>`.
pub struct TreePanel {
    mode: TreeMode,
    file_tree: TreeView<FileTreeData>,
    root: PathBuf,
}

impl TreePanel {
    /// Create a new tree panel rooted at `root`.
    pub fn new(root: &Path) -> Self {
        let data = FileTreeData::new(root, 8).unwrap_or_else(|_| {
            FileTreeData::new(Path::new("."), 1).unwrap_or_else(|_| {
                // Fallback: this should not fail for "."
                panic!("cannot create FileTreeData")
            })
        });
        Self {
            mode: TreeMode::Files,
            file_tree: TreeView::new(data),
            root: root.to_path_buf(),
        }
    }

    /// Currently selected file path, if any.
    pub fn selected_path(&self) -> Option<&Path> {
        self.file_tree.selected_node().map(|p| p.as_path())
    }

    /// Whether the selected node is a directory.
    pub fn selected_is_dir(&self) -> bool {
        self.file_tree
            .selected_node()
            .map(|p| self.file_tree.data().is_dir(p))
            .unwrap_or(false)
    }

    /// Refresh tree data from disk.
    pub fn refresh(&mut self) {
        if let Ok(data) = FileTreeData::new(&self.root, 8) {
            self.file_tree.set_data(data);
        }
    }

    /// Cycle tree mode.
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            TreeMode::Files => TreeMode::Git,
            TreeMode::Git => TreeMode::Symbols,
            TreeMode::Symbols => TreeMode::Packages,
            TreeMode::Packages => TreeMode::Files,
        };
    }

    /// Current tree mode.
    pub fn mode(&self) -> TreeMode {
        self.mode
    }
}

impl Widget for TreePanel {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        self.file_tree.render(surface, focused);
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        self.file_tree.handle_key(key)
    }

    fn focusable(&self) -> bool {
        true
    }
}
