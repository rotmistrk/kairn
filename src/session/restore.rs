//! Session restore — apply saved state to workspace.

use super::restore_split;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use git2::Repository;
use txv_widgets::tiled_workspace::workspace_state::WorkspaceState;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::desktop::{close_tab_by_title, insert_tab, LayoutMode, SlotId};
use crate::kiro_registry::KiroTabRegistry;
use crate::settings::EditorSettings;
use crate::views::editor::EditorView;
use crate::views::terminal::new_kiro_terminal_argv;
use crate::views::tree::FileTreeView;

use super::schema::{KiroSessionState, SessionState};

/// Apply restored layout mode and panel proportions to the desktop.
pub fn restore_session(desktop: &mut TiledWorkspace, state: &SessionState) {
    desktop.set_layout_mode(match state.layout.as_str() {
        "wide" => LayoutMode::Wide,
        "tall" => LayoutMode::Narrow,
        _ => LayoutMode::Auto,
    });
    if !state.wide_proportions.is_empty() || !state.narrow_proportions.is_empty() {
        let ws_state = WorkspaceState::new(
            state.wide_proportions.clone(),
            state.narrow_proportions.clone(),
            state.hidden_panels.clone(),
        );
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
    max_tabs: usize,
) {
    if state.editor_tabs.is_empty() {
        return;
    }
    close_tab_by_title(desktop, SlotId::Center, "Welcome");

    let second_set: HashSet<usize> = state
        .split
        .as_ref()
        .map(|s| s.second_tabs.iter().copied().collect())
        .unwrap_or_default();

    open_first_panel_tabs(
        desktop,
        state,
        &second_set,
        root_dir,
        editor_defaults,
        syntax_theme,
        max_tabs,
    );
    set_first_panel_active(desktop, state);

    if let Some(ref split) = state.split {
        restore_split::restore_split(desktop, state, split, root_dir, editor_defaults, syntax_theme);
    }
    restore_unfolded_dirs(desktop, state, root_dir);
}

fn open_first_panel_tabs(
    desktop: &mut TiledWorkspace,
    state: &SessionState,
    second_set: &HashSet<usize>,
    root_dir: &Path,
    editor_defaults: &EditorSettings,
    syntax_theme: &str,
    max_tabs: usize,
) {
    let mut count = 0;
    for (i, tab) in state.editor_tabs.iter().enumerate() {
        if second_set.contains(&i) {
            continue;
        }
        if max_tabs > 0 && count >= max_tabs {
            break;
        }
        open_tab_in_panel(desktop, tab, root_dir, editor_defaults, syntax_theme);
        count += 1;
    }
}

fn set_first_panel_active(desktop: &mut TiledWorkspace, state: &SessionState) {
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
    let path = PathBuf::from(&tab.path);
    // Reject relative paths — session must store absolute
    let path = if path.is_absolute() {
        path
    } else {
        log::warn!("session restore: rejecting relative path: {}", tab.path);
        return;
    };
    if !path.is_file() {
        return;
    }
    let title = path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
    let mut editor = EditorView::open_with_theme(&path, editor_defaults, syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(&path, editor_defaults));
    editor.set_root_dir(discover_root_for(&path, root_dir));
    editor.goto(tab.line, tab.col);
    insert_tab(desktop, SlotId::Center, title, Box::new(editor));
}

/// Find the git root for a path, falling back to the given default root.
pub(super) fn discover_root_for(path: &Path, fallback: &Path) -> PathBuf {
    Repository::discover(path.parent().unwrap_or(fallback))
        .ok()
        .and_then(|repo| repo.workdir().map(|w| w.to_path_buf()))
        .unwrap_or_else(|| fallback.to_path_buf())
}

/// Restore kiro tabs from saved session state.
pub fn restore_kiro_tabs(
    desktop: &mut TiledWorkspace,
    sessions: &[KiroSessionState],
    root_dir: &Path,
    registry: &mut KiroTabRegistry,
    kiro: &crate::settings::KiroLaunchSettings,
) {
    for (i, session) in sessions.iter().enumerate() {
        let mut argv: Vec<String> = kiro.cmd.clone();
        // First session gets resume-first args, rest get resume-rest
        let extra = if i == 0 {
            &kiro.resume_first
        } else {
            &kiro.resume_rest
        };
        argv.extend(extra.iter().cloned());
        // Ensure --agent is present (default to kairn)
        if !argv.iter().any(|a| a.starts_with("--agent")) {
            argv.push("--agent=kairn".to_string());
        }
        let term = new_kiro_terminal_argv(&argv, root_dir);
        insert_tab(desktop, SlotId::Tools, &session.name, term);
        registry.register_with_id(&session.name, session.session_id.clone());
    }
}
