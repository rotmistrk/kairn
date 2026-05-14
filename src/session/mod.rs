//! Session persistence — save/restore workspace state.

pub mod schema;

use std::path::Path;

use crate::kiro_registry::KiroTabRegistry;
use crate::layout_group::{LayoutGroup, LayoutMode, SlotId};
use crate::settings::EditorSettings;
use crate::views::editor::EditorView;
use crate::views::terminal::new_kiro_terminal_with_resume;
use crate::views::tree::FileTreeView;

use schema::{EditorTabState, KiroSessionState, SessionState, SESSION_VERSION};

const STATE_FILE: &str = ".kairn.state";

/// Collect current state from the desktop and save to `.kairn.state`.
pub fn save_session(desktop: &mut LayoutGroup, root_dir: &Path, kiro_registry: &KiroTabRegistry) {
    let state = collect_state(desktop, root_dir, kiro_registry);
    if state.editor_tabs.is_empty() && state.unfolded_dirs.is_empty() && state.kiro_sessions.is_empty() {
        return;
    }
    let path = root_dir.join(STATE_FILE);
    let json = match serde_json::to_string_pretty(&state) {
        Ok(j) => j,
        Err(e) => {
            log::warn!("session: failed to serialize: {e}");
            return;
        }
    };
    if let Err(e) = std::fs::write(&path, json) {
        log::warn!("session: failed to write {}: {e}", path.display());
    }
}

/// Load session state from `.kairn.state`. Returns None if missing/corrupt.
pub fn load_session(root_dir: &Path) -> Option<SessionState> {
    let path = root_dir.join(STATE_FILE);
    let content = std::fs::read_to_string(&path).ok()?;
    let state: SessionState = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            log::info!("session: ignoring corrupt state file: {e}");
            return None;
        }
    };
    if state.version != SESSION_VERSION {
        log::info!("session: version mismatch (got {})", state.version);
        return None;
    }
    Some(state)
}

/// Apply restored layout mode to the desktop.
pub fn restore_session(desktop: &mut LayoutGroup, state: &SessionState) {
    desktop.layout_mode = match state.layout.as_str() {
        "wide" => LayoutMode::Wide,
        "tall" => LayoutMode::Tall,
        _ => LayoutMode::Auto,
    };
}

/// Restore editor tabs and unfolded directories.
pub fn restore_tabs(
    desktop: &mut LayoutGroup,
    state: &SessionState,
    root_dir: &Path,
    editor_defaults: &EditorSettings,
    syntax_theme: &str,
) {
    // Restore editor tabs
    if !state.editor_tabs.is_empty() {
        desktop.close_tab_by_title(SlotId::Center, "Welcome");
        for tab in &state.editor_tabs {
            let path = root_dir.join(&tab.path);
            if !path.is_file() {
                continue;
            }
            let mut editor = EditorView::open_with_theme(&path, editor_defaults, syntax_theme)
                .unwrap_or_else(|_| EditorView::new_file(&path, editor_defaults));
            editor.set_root_dir(root_dir.to_path_buf());
            editor.goto(tab.line, tab.col);
            let title = tab.path.clone();
            desktop.insert_tab(SlotId::Center, &title, Box::new(editor));
        }
        // Restore active tab index
        let count = desktop.panel(SlotId::Center).tab_count();
        if state.active_tab < count {
            desktop.set_active_tab(SlotId::Center, state.active_tab);
        }
    }
    // Restore unfolded directories
    if !state.unfolded_dirs.is_empty() {
        let paths: Vec<_> = state.unfolded_dirs.iter().map(|d| root_dir.join(d)).collect();
        let panel = desktop.panel_mut(SlotId::Left);
        if let Some(view) = panel.view_at_mut(0) {
            if let Some(any) = view.as_any_mut() {
                if let Some(tree) = any.downcast_mut::<FileTreeView>() {
                    tree.expand_paths(&paths);
                }
            }
        }
    }
}

/// Restore kiro tabs from saved session state.
pub fn restore_kiro_tabs(
    desktop: &mut LayoutGroup,
    sessions: &[KiroSessionState],
    root_dir: &Path,
    registry: &mut KiroTabRegistry,
) {
    for session in sessions {
        let resume_id = session.session_id.as_deref();
        let term = new_kiro_terminal_with_resume(Some("kairn"), resume_id, root_dir);
        desktop.insert_tab(SlotId::Right, &session.name, term);
        registry.register_with_id(&session.name, session.session_id.clone());
    }
}

fn collect_state(desktop: &mut LayoutGroup, root_dir: &Path, kiro_registry: &KiroTabRegistry) -> SessionState {
    let layout = match desktop.layout_mode {
        LayoutMode::Auto => "auto",
        LayoutMode::Wide => "wide",
        LayoutMode::Tall => "tall",
    };
    let editor_tabs = collect_editor_tabs(desktop, root_dir);
    let active_tab = desktop.panel(SlotId::Center).active_index();
    let unfolded_dirs = collect_unfolded_dirs(desktop, root_dir);
    let kiro_sessions = kiro_registry.to_state();

    SessionState {
        version: SESSION_VERSION,
        layout: layout.to_string(),
        left_width: desktop.left_width,
        right_width: desktop.right_width,
        active_tab,
        editor_tabs,
        unfolded_dirs,
        kiro_sessions,
    }
}

fn collect_editor_tabs(desktop: &mut LayoutGroup, root_dir: &Path) -> Vec<EditorTabState> {
    let panel = desktop.panel_mut(SlotId::Center);
    let count = panel.tab_count();
    let mut tabs = Vec::new();
    for i in 0..count {
        let Some(view) = panel.view_at_mut(i) else {
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
        let rel = editor
            .path()
            .strip_prefix(root_dir)
            .unwrap_or(editor.path())
            .to_string_lossy()
            .to_string();
        tabs.push(EditorTabState {
            path: rel,
            line: editor.editor.cursor_line as u32,
            col: editor.editor.cursor_col as u32,
        });
    }
    tabs
}

fn collect_unfolded_dirs(desktop: &mut LayoutGroup, root_dir: &Path) -> Vec<String> {
    let panel = desktop.panel_mut(SlotId::Left);
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
