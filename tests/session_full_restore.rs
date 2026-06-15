//! Full session save/restore integration — verifies cursor positions survive.

use kairn::build_desktop;
use kairn::git_keys::GitKeys;
use kairn::kiro_registry::KiroTabRegistry;
use kairn::session;
use kairn::session::schema::SESSION_VERSION;
use kairn::settings::EditorSettings;
use kairn::slots::SlotId;
use kairn::views::editor::{EditorView, EditorViewDiffExt, EditorViewExt};

#[test]
fn session_save_restore_preserves_cursor_positions() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();

    // Create test files with multiple lines
    std::fs::write(root.join("a.rs"), "line1\nline2\nline3\nline4\nline5\n").unwrap();
    std::fs::write(root.join("b.rs"), "fn main() {\n    let x = 1;\n    let y = 2;\n}\n").unwrap();

    let defaults = EditorSettings::default();
    let git_keys = GitKeys::default();

    // Build desktop, open 2 files with specific cursor positions
    let mut desktop = build_desktop::build_workspace(&root, git_keys.clone());
    let mut ed_a = kairn::views::editor::build::open(&root.join("a.rs"), &defaults).unwrap();
    ed_a.set_root_dir(root.clone());
    ed_a.goto(3, 2); // line 3, col 2
    desktop.insert_tab(SlotId::Center as usize, "a.rs", Box::new(ed_a));

    let mut ed_b = kairn::views::editor::build::open(&root.join("b.rs"), &defaults).unwrap();
    ed_b.set_root_dir(root.clone());
    ed_b.goto(1, 8); // line 1, col 8
    desktop.insert_tab(SlotId::Center as usize, "b.rs", Box::new(ed_b));

    // Save session
    let registry = KiroTabRegistry::default();
    session::save_session(&mut desktop, &root, &registry, &[]).unwrap();

    // Load session
    let loaded = session::load_session(&root).unwrap();
    assert_eq!(loaded.version(), SESSION_VERSION);
    assert_eq!(loaded.editor_tabs().len(), 2);

    // Verify cursor positions were saved
    let tab_a = loaded
        .editor_tabs()
        .iter()
        .find(|t| t.path().ends_with("a.rs"))
        .unwrap();
    assert_eq!(tab_a.line(), 3, "a.rs cursor line should be 3");
    assert_eq!(tab_a.col(), 2, "a.rs cursor col should be 2");

    let tab_b = loaded
        .editor_tabs()
        .iter()
        .find(|t| t.path().ends_with("b.rs"))
        .unwrap();
    assert_eq!(tab_b.line(), 1, "b.rs cursor line should be 1");
    assert_eq!(tab_b.col(), 8, "b.rs cursor col should be 8");

    // Restore into a fresh desktop
    let mut desktop2 = build_desktop::build_workspace(&root, git_keys);
    session::restore_tabs(&mut desktop2, &loaded, &root, &defaults, "base16-eighties.dark", 0);

    // Verify both files are open
    let panel = desktop2.panel(SlotId::Center as usize).unwrap();
    assert_eq!(panel.tab_count(), 2, "should have 2 tabs after restore");
}
