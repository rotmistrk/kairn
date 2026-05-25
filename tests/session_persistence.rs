//! Tests for session persistence (save/restore).

use kairn::kiro_registry::KiroTabRegistry;
use kairn::session;
use kairn::session::schema::{EditorTabState, SessionState, SESSION_VERSION};
use kairn::settings::EditorSettings;
use kairn::slots::{LayoutMode, SlotId};
use kairn::views::editor::EditorView;

#[test]
fn save_and_load_session_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Create a test file
    std::fs::write(root.join("hello.rs"), "fn main() {}\n").unwrap();

    // Build a desktop with an editor tab
    let mut desktop =
        kairn::build_desktop::build_workspace(&std::path::PathBuf::from("."), kairn::settings::GitKeys::default());
    let defaults = EditorSettings::default();
    let path = root.join("hello.rs");
    let mut editor = EditorView::open(&path, &defaults).unwrap();
    editor.set_root_dir(root.clone());
    editor.goto(0, 5);
    desktop.insert_tab(SlotId::Center as usize, "hello.rs", Box::new(editor));
    desktop.set_layout_mode(LayoutMode::Wide);

    // Save
    session::save_session(&mut desktop, &root, &KiroTabRegistry::default());

    // Verify file exists
    let state_path = root.join(".kairn.state");
    assert!(state_path.exists());

    // Load
    let loaded = session::load_session(&root).unwrap();
    assert_eq!(loaded.version, SESSION_VERSION);
    assert_eq!(loaded.layout, "wide");
    assert_eq!(loaded.editor_tabs.len(), 1);
    assert_eq!(loaded.editor_tabs[0].path, "hello.rs");
    assert_eq!(loaded.editor_tabs[0].line, 0);
    assert_eq!(loaded.editor_tabs[0].col, 5);
}

#[test]
fn load_missing_state_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(session::load_session(tmp.path()).is_none());
}

#[test]
fn load_corrupt_state_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join(".kairn.state"), "not json").unwrap();
    assert!(session::load_session(tmp.path()).is_none());
}

#[test]
fn load_wrong_version_returns_none() {
    let tmp = tempfile::tempdir().unwrap();
    let state = SessionState {
        version: 999,
        ..SessionState::default()
    };
    let json = serde_json::to_string(&state).unwrap();
    std::fs::write(tmp.path().join(".kairn.state"), json).unwrap();
    assert!(session::load_session(tmp.path()).is_none());
}

#[test]
fn restore_tabs_opens_editors() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::write(root.join("foo.rs"), "let x = 1;\nlet y = 2;\n").unwrap();

    let state = SessionState {
        version: SESSION_VERSION,
        layout: "auto".to_string(),
        wide_proportions: Vec::new(),
        narrow_proportions: Vec::new(),
        hidden_panels: Vec::new(),
        active_tab: 0,
        editor_tabs: vec![EditorTabState {
            path: "foo.rs".to_string(),
            line: 1,
            col: 4,
        }],
        unfolded_dirs: Vec::new(),
        kiro_sessions: Vec::new(),
        split: None,
    };

    let mut desktop =
        kairn::build_desktop::build_workspace(&std::path::PathBuf::from("."), kairn::settings::GitKeys::default());
    let defaults = EditorSettings::default();
    session::restore_tabs(&mut desktop, &state, &root, &defaults, "base16-eighties.dark");

    assert_eq!(desktop.panel(SlotId::Center as usize).unwrap().tab_count(), 1);
    assert_eq!(
        desktop.panel(SlotId::Center as usize).unwrap().tab_title(0),
        Some("foo.rs")
    );
}

#[test]
fn restore_skips_missing_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    let state = SessionState {
        version: SESSION_VERSION,
        layout: "auto".to_string(),
        wide_proportions: Vec::new(),
        narrow_proportions: Vec::new(),
        hidden_panels: Vec::new(),
        active_tab: 0,
        editor_tabs: vec![EditorTabState {
            path: "nonexistent.rs".to_string(),
            line: 0,
            col: 0,
        }],
        unfolded_dirs: Vec::new(),
        kiro_sessions: Vec::new(),
        split: None,
    };

    let mut desktop =
        kairn::build_desktop::build_workspace(&std::path::PathBuf::from("."), kairn::settings::GitKeys::default());
    let defaults = EditorSettings::default();
    session::restore_tabs(&mut desktop, &state, &root, &defaults, "base16-eighties.dark");

    // No tabs opened since file doesn't exist
    assert_eq!(desktop.panel(SlotId::Center as usize).unwrap().tab_count(), 0);
}
