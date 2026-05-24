//! Workspace builder — constructs the initial kairn layout.

use std::path::Path;
use std::sync::Arc;

use txv_widgets::tiled_workspace::types::{PanelConfig, PanelPosition, SplitNode};
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::git_watcher::{GitWatcher, WatchHandle};
use crate::settings::GitKeys;
use crate::slots::{insert_tab, SlotId, PANEL_COUNT};
use crate::views::git_changes::GitChangesView;
use crate::views::terminal::new_shell_terminal;
use crate::views::todo_tree::TodoTreeView;
use crate::views::tree::FileTreeView;
use crate::views::welcome::WelcomeView;

pub fn create_workspace_shell() -> TiledWorkspace {
    let configs = vec![
        PanelConfig::fixed("Files", PanelPosition::Left),
        PanelConfig {
            splittable: true,
            ..PanelConfig::new("Editor", PanelPosition::Center)
        },
        PanelConfig {
            splittable: true,
            ..PanelConfig::new("Tools", PanelPosition::Right)
        },
    ];
    let wide_layout = SplitNode::h(vec![
        (0.2, SplitNode::leaf(0)),
        (0.4, SplitNode::leaf(1)),
        (0.4, SplitNode::leaf(2)),
    ]);
    let narrow_layout = SplitNode::v(vec![
        (
            0.7,
            SplitNode::h(vec![(0.2, SplitNode::leaf(0)), (0.8, SplitNode::leaf(1))]),
        ),
        (0.3, SplitNode::leaf(2)),
    ]);
    let mut ws = TiledWorkspace::new(configs, wide_layout, narrow_layout, 300);
    ws.set_narrow_threshold(200);
    ws.set_handle_keys(false);
    ws.set_v_divider_gaps(false);

    // Customize keymap: Ctrl+Shift+Up/Down for dropdown navigation
    use txv_core::event::{KeyCode, KeyEvent, KeyMod};
    let ctrl_shift = |code| KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: true,
            shift: true,
            alt: false,
        },
    };
    let mut km = ws.keymap().clone();
    km.tab_dropdown_up = ctrl_shift(KeyCode::Up);
    km.tab_dropdown_down = ctrl_shift(KeyCode::Down);
    // Remove focus_up/down (kairn uses F-keys for panel focus)
    km.focus_up = KeyEvent {
        code: KeyCode::F(127),
        modifiers: KeyMod::default(),
    };
    km.focus_down = KeyEvent {
        code: KeyCode::F(127),
        modifiers: KeyMod::default(),
    };
    ws.set_keymap(km);
    for i in 0..PANEL_COUNT {
        if let Some(panel) = ws.panel_mut(i) {
            panel.bar_mut().set_handle_keys(false);
        }
    }
    ws.focus_panel(0);
    ws
}

/// Build the standard kairn workspace with tree, welcome, and terminal.
pub fn build_workspace(root_dir: &Path, git_keys: GitKeys) -> TiledWorkspace {
    let mut ws = create_workspace_shell();

    let watcher = GitWatcher::new(root_dir).map(Arc::new);
    let tree_handle = watcher.as_ref().map(|w| WatchHandle::new(w.clone()));
    let git_handle = watcher.as_ref().map(|w| WatchHandle::new(w.clone()));

    let tree = FileTreeView::new(root_dir.to_path_buf(), tree_handle);
    insert_tab(&mut ws, SlotId::Left, "Files", Box::new(tree));

    let git_panel = GitChangesView::new(root_dir.to_path_buf(), git_handle, git_keys);
    insert_tab(&mut ws, SlotId::Left, "Git", Box::new(git_panel));

    let todo_panel = TodoTreeView::new(root_dir);
    insert_tab(&mut ws, SlotId::Left, "Todo", Box::new(todo_panel));

    if let Some(panel) = ws.panel_mut(SlotId::Left as usize) {
        panel.set_active(0);
    }

    let welcome = WelcomeView::new(root_dir.to_path_buf());
    insert_tab(&mut ws, SlotId::Center, "Welcome", Box::new(welcome));

    let term = new_shell_terminal();
    insert_tab(&mut ws, SlotId::Tools, "Shell:0", term);

    ws
}

/// Deprecated alias — use [`build_workspace`] instead.
#[deprecated(note = "use build_workspace")]
pub fn build_desktop(root_dir: &Path, git_keys: GitKeys) -> TiledWorkspace {
    build_workspace(root_dir, git_keys)
}
