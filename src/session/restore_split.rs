//! Split-related session restoration.

use super::restore::discover_root_for;
use std::path::{Path, PathBuf};

use txv_widgets::tab_panel::TabPanel;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::desktop::SlotId;
use crate::settings::EditorSettings;
use crate::views::editor::EditorView;

use super::schema::SessionState;

pub(super) fn restore_split(
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

    open_second_panel_tabs(desktop, state, split, root_dir, editor_defaults, syntax_theme);
    set_second_panel_active(desktop, split);
}

fn open_second_panel_tabs(
    desktop: &mut TiledWorkspace,
    state: &SessionState,
    split: &super::schema::SplitState,
    root_dir: &Path,
    editor_defaults: &EditorSettings,
    syntax_theme: &str,
) {
    for &i in &split.second_tabs {
        let Some(tab) = state.editor_tabs.get(i) else {
            continue;
        };
        let path = PathBuf::from(&tab.path);
        let path = if path.is_absolute() {
            path
        } else {
            log::warn!("session restore: rejecting relative path in split: {}", tab.path);
            continue;
        };
        if !path.is_file() {
            continue;
        }
        let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        let mut editor = EditorView::open_with_theme(&path, editor_defaults, syntax_theme)
            .unwrap_or_else(|_| EditorView::new_file(&path, editor_defaults));
        editor.set_root_dir(discover_root_for(&path, root_dir));
        editor.goto(tab.line, tab.col);
        insert_into_second_panel(desktop, editor, title);
    }
}

fn insert_into_second_panel(desktop: &mut TiledWorkspace, editor: EditorView, title: &str) {
    if desktop
        .split_panel(SlotId::Center as usize)
        .is_none_or(|sp| sp.child_count() <= 1)
    {
        desktop.split_in_place(Box::new(editor), title);
        return;
    }
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    let Some(child) = sp.child_mut(1) else {
        return;
    };
    let Some(tp) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
        return;
    };
    tp.insert_tab(title, Box::new(editor));
}

fn set_second_panel_active(desktop: &mut TiledWorkspace, split: &super::schema::SplitState) {
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    if sp.child_count() <= 1 {
        return;
    }
    let Some(child) = sp.child_mut(1) else {
        return;
    };
    if let Some(tp) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) {
        if split.active_second < tp.tab_count() {
            tp.set_active(split.active_second);
        }
    }
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    sp.set_focused(split.focused);
}
