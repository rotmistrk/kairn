//! Session restore — apply saved state to workspace.

use std::path::Path;

use crate::desktop::{close_tab_by_title, insert_tab, SlotId};
use crate::kiro_registry::KiroTabRegistry;
use crate::settings::EditorSettings;
use crate::views::editor::EditorView;
use crate::views::terminal::new_kiro_terminal_with_resume;
use crate::views::tree::FileTreeView;
use txv_widgets::tiled_workspace::TiledWorkspace;

use super::schema::{KiroSessionState, SessionState};

/// Apply restored layout mode and panel proportions to the desktop.
pub fn restore_session(desktop: &mut TiledWorkspace, state: &SessionState) {
    desktop.set_layout_mode(match state.layout.as_str() {
        "wide" => crate::desktop::LayoutMode::Wide,
        "tall" => crate::desktop::LayoutMode::Narrow,
        _ => crate::desktop::LayoutMode::Auto,
    });
    if !state.wide_proportions.is_empty() || !state.narrow_proportions.is_empty() {
        let ws_state = txv_widgets::tiled_workspace::types::WorkspaceState {
            wide_proportions: state.wide_proportions.clone(),
            narrow_proportions: state.narrow_proportions.clone(),
            hidden: state.hidden_panels.clone(),
        };
        desktop.restore_state(&ws_state);
    }
}

/// Restore editor tabs, splits, and unfolded directories.
pub fn restore_tabs(
    desktop: &mut TiledWorkspace,
    state: &SessionState,
    root_dir: &Path,
    editor_defaults: &EditorSettings,
    syntax_theme: &str,
) {
    if state.editor_tabs.is_empty() {
        return;
    }
    close_tab_by_title(desktop, SlotId::Center, "Welcome");

    let second_set: std::collections::HashSet<usize> = state
        .split
        .as_ref()
        .map(|s| s.second_tabs.iter().copied().collect())
        .unwrap_or_default();

    // Open tabs in first subpanel
    for (i, tab) in state.editor_tabs.iter().enumerate() {
        if second_set.contains(&i) {
            continue;
        }
        open_tab_in_panel(desktop, tab, root_dir, editor_defaults, syntax_theme);
    }

    // Set active tab in first subpanel
    if let Some(split) = &state.split {
        if let Some(panel) = desktop.panel_mut(SlotId::Center as usize) {
            if split.active_first < panel.tab_count() {
                panel.set_active(split.active_first);
            }
        }
    } else if let Some(panel) = desktop.panel_mut(SlotId::Center as usize) {
        if state.active_tab < panel.tab_count() {
            panel.set_active(state.active_tab);
        }
    }

    // Restore split if present
    if let Some(ref split) = state.split {
        restore_split(desktop, state, split, root_dir, editor_defaults, syntax_theme);
    }

    // Restore unfolded directories
    restore_unfolded_dirs(desktop, state, root_dir);
}

fn restore_split(
    desktop: &mut TiledWorkspace,
    state: &SessionState,
    split: &super::schema::SplitState,
    root_dir: &Path,
    editor_defaults: &EditorSettings,
    syntax_theme: &str,
) {
    use txv_widgets::tiled_workspace::types::SplitDir;
    let dir = match split.direction.as_str() {
        "vertical" => SplitDir::Vertical,
        _ => SplitDir::Horizontal,
    };
    if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
        sp.set_direction(dir);
    }

    for &i in &split.second_tabs {
        let Some(tab) = state.editor_tabs.get(i) else {
            continue;
        };
        let path = root_dir.join(&tab.path);
        if !path.is_file() {
            continue;
        }
        let mut editor = EditorView::open_with_theme(&path, editor_defaults, syntax_theme)
            .unwrap_or_else(|_| EditorView::new_file(&path, editor_defaults));
        editor.set_root_dir(root_dir.to_path_buf());
        editor.goto(tab.line, tab.col);
        let title = tab.path.clone();

        if desktop
            .split_panel(SlotId::Center as usize)
            .is_none_or(|sp| sp.child_count() <= 1)
        {
            desktop.split_in_place(Box::new(editor), &title);
        } else if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
            if let Some(child) = sp.child_mut(1) {
                if let Some(tp) = child
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<txv_widgets::tab_panel::TabPanel>())
                {
                    tp.insert_tab(&title, Box::new(editor));
                }
            }
        }
    }

    // Set active tab in second subpanel and focused subpanel
    if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
        if sp.child_count() > 1 {
            if let Some(child) = sp.child_mut(1) {
                if let Some(tp) = child
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<txv_widgets::tab_panel::TabPanel>())
                {
                    if split.active_second < tp.tab_count() {
                        tp.set_active(split.active_second);
                    }
                }
            }
            sp.set_focused(split.focused);
        }
    }
}

fn restore_unfolded_dirs(desktop: &mut TiledWorkspace, state: &SessionState, root_dir: &Path) {
    if state.unfolded_dirs.is_empty() {
        return;
    }
    let paths: Vec<_> = state.unfolded_dirs.iter().map(|d| root_dir.join(d)).collect();
    let Some(panel) = desktop.panel_mut(SlotId::Left as usize) else {
        return;
    };
    if let Some(view) = panel.view_at_mut(0) {
        if let Some(any) = view.as_any_mut() {
            if let Some(tree) = any.downcast_mut::<FileTreeView>() {
                tree.expand_paths(&paths);
            }
        }
    }
}

fn open_tab_in_panel(
    desktop: &mut TiledWorkspace,
    tab: &super::schema::EditorTabState,
    root_dir: &Path,
    editor_defaults: &EditorSettings,
    syntax_theme: &str,
) {
    let path = root_dir.join(&tab.path);
    if !path.is_file() {
        return;
    }
    let mut editor = EditorView::open_with_theme(&path, editor_defaults, syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(&path, editor_defaults));
    editor.set_root_dir(root_dir.to_path_buf());
    editor.goto(tab.line, tab.col);
    insert_tab(desktop, SlotId::Center, &tab.path, Box::new(editor));
}

/// Restore kiro tabs from saved session state.
pub fn restore_kiro_tabs(
    desktop: &mut TiledWorkspace,
    sessions: &[KiroSessionState],
    root_dir: &Path,
    registry: &mut KiroTabRegistry,
) {
    for session in sessions {
        let resume_id = session.session_id.as_deref();
        let term = new_kiro_terminal_with_resume(Some("kairn"), resume_id, root_dir);
        insert_tab(desktop, SlotId::Tools, &session.name, term);
        registry.register_with_id(&session.name, session.session_id.clone());
    }
}
