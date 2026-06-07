//! Session save — collect workspace state for persistence.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::desktop::{LayoutMode, SlotId};
use crate::kiro_registry::KiroTabRegistry;
use crate::views::editor::EditorView;
use crate::views::tree::FileTreeView;

use super::schema::{EditorTabState, SessionState, SplitState, SESSION_VERSION};

pub(super) fn collect_state(
    desktop: &mut TiledWorkspace,
    root_dir: &Path,
    kiro_registry: &KiroTabRegistry,
    roots: &[&Path],
) -> SessionState {
    let layout = match desktop.layout_mode() {
        LayoutMode::Auto => "auto",
        LayoutMode::Wide => "wide",
        LayoutMode::Narrow => "tall",
    };
    let (editor_tabs, split) = collect_editor_tabs_and_split(desktop, root_dir);
    let active_tab = desktop.panel(SlotId::Center as usize).map_or(0, |p| p.active_index());
    let unfolded_dirs = collect_unfolded_dirs(desktop, root_dir);
    let kiro_sessions = kiro_registry.to_state();
    let ws_state = desktop.save_state();
    // Save additional roots (skip primary which is root_dir itself).
    let saved_roots: Vec<String> = roots
        .iter()
        .filter(|r| **r != root_dir)
        .map(|r| r.to_string_lossy().to_string())
        .collect();

    SessionState {
        version: SESSION_VERSION,
        layout: layout.to_string(),
        wide_proportions: ws_state.wide_proportions().to_vec(),
        narrow_proportions: ws_state.narrow_proportions().to_vec(),
        hidden_panels: ws_state.hidden().to_vec(),
        active_tab,
        editor_tabs,
        unfolded_dirs,
        kiro_sessions,
        split,
        roots: saved_roots,
    }
}

fn collect_editor_tabs_and_split(
    desktop: &mut TiledWorkspace,
    root_dir: &Path,
) -> (Vec<EditorTabState>, Option<SplitState>) {
    use txv_widgets::tiled_workspace::types::SplitDir;

    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return (Vec::new(), None);
    };
    let is_split = sp.child_count() > 1;
    let focused_sub = sp.focused_index();

    let first_tabs = collect_tabs_from_child(sp.child_mut(0), root_dir);
    let active_first = child_active_index(sp.child_mut(0));

    if !is_split {
        return (first_tabs, None);
    }

    let second_tabs = collect_tabs_from_child(sp.child_mut(1), root_dir);
    let active_second = child_active_index(sp.child_mut(1));
    let dir = match sp.direction() {
        SplitDir::Horizontal => "horizontal",
        SplitDir::Vertical => "vertical",
    };

    let first_count = first_tabs.len();
    let second_indices: Vec<usize> = (first_count..first_count + second_tabs.len()).collect();
    let mut all_tabs = first_tabs;
    all_tabs.extend(second_tabs);

    let split = SplitState {
        direction: dir.to_string(),
        second_tabs: second_indices,
        active_first,
        active_second,
        focused: focused_sub,
    };
    (all_tabs, Some(split))
}

fn child_active_index(child: Option<&mut Box<dyn View>>) -> usize {
    use txv_widgets::tab_panel::TabPanel;
    child
        .and_then(|c| c.as_any_mut())
        .and_then(|a| a.downcast_ref::<TabPanel>())
        .map_or(0, |tp| tp.active_index())
}

fn collect_tabs_from_child(child: Option<&mut Box<dyn View>>, _root_dir: &Path) -> Vec<EditorTabState> {
    use txv_widgets::tab_panel::TabPanel;
    let Some(child) = child else {
        return Vec::new();
    };
    let Some(tp) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
        return Vec::new();
    };
    let mut tabs = Vec::new();
    for i in 0..tp.tab_count() {
        let Some(view) = tp.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(editor) = any.downcast_ref::<EditorView>() else {
            continue;
        };
        let path_str = editor.path().to_string_lossy().to_string();
        if path_str.starts_with('[') {
            continue;
        }
        // Store absolute path
        tabs.push(EditorTabState {
            path: path_str,
            line: editor.editor.cursor_line() as u32,
            col: editor.editor.cursor_col() as u32,
        });
    }
    tabs
}

fn collect_unfolded_dirs(desktop: &mut TiledWorkspace, root_dir: &Path) -> Vec<String> {
    let Some(panel) = desktop.panel_mut(SlotId::Left as usize) else {
        return Vec::new();
    };
    let Some(view) = panel.view_at_mut(0) else {
        return Vec::new();
    };
    let Some(any) = view.as_any_mut() else {
        return Vec::new();
    };
    let Some(tree) = any.downcast_mut::<FileTreeView>() else {
        return Vec::new();
    };
    tree.expanded_paths()
        .into_iter()
        .filter_map(|p| p.strip_prefix(root_dir).ok().map(|r| r.to_string_lossy().to_string()))
        .collect()
}
