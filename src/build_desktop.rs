//! Desktop builder — constructs the initial kairn layout.

use std::path::Path;

use crate::desktop::{SlotId, SlottedDesktop};
use crate::views::terminal::new_shell_terminal;
use crate::views::tree::FileTreeView;
use crate::views::welcome::WelcomeView;

/// Build the standard kairn desktop with tree and terminal.
pub fn build_desktop(root_dir: &Path) -> SlottedDesktop {
    let mut desktop = SlottedDesktop::new();
    let tree = FileTreeView::new(root_dir.to_path_buf());
    desktop.insert_tab(SlotId::Left, "Files", Box::new(tree));
    let welcome = WelcomeView::new();
    desktop.insert_tab(SlotId::Center, "Welcome", Box::new(welcome));
    let term = new_shell_terminal();
    desktop.insert_tab(SlotId::Right, "Shell:0", term);
    desktop
}
