//! Desktop builder — constructs the initial kairn layout.

use std::path::Path;
use std::sync::Arc;

use crate::desktop::{SlotId, SlottedDesktop};
use crate::git_watcher::{GitWatcher, WatchHandle};
use crate::settings::GitKeys;
use crate::views::git_changes::GitChangesView;
use crate::views::terminal::new_shell_terminal;
use crate::views::tree::FileTreeView;
use crate::views::welcome::WelcomeView;

/// Build the standard kairn desktop with tree and terminal.
pub fn build_desktop(root_dir: &Path, git_keys: GitKeys) -> SlottedDesktop {
    let mut desktop = SlottedDesktop::new();

    // Shared git watcher — both tree and git panel react to same events
    let watcher = GitWatcher::new(root_dir).map(Arc::new);
    let tree_handle = watcher.as_ref().map(|w| WatchHandle::new(w.clone()));
    let git_handle = watcher.as_ref().map(|w| WatchHandle::new(w.clone()));

    let tree = FileTreeView::new(root_dir.to_path_buf(), tree_handle);
    desktop.insert_tab(SlotId::Left, "Files", Box::new(tree));

    let git_panel = GitChangesView::new(root_dir.to_path_buf(), git_handle, git_keys);
    desktop.insert_tab(SlotId::Left, "Git", Box::new(git_panel));

    // Keep "Files" as the initially active tab
    desktop.set_active_tab(SlotId::Left, 0);

    let welcome = WelcomeView::new();
    desktop.insert_tab(SlotId::Center, "Welcome", Box::new(welcome));
    let term = new_shell_terminal();
    desktop.insert_tab(SlotId::Right, "Shell:0", term);
    desktop
}
