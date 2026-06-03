//! Test that CM_ROOTS_CHANGED broadcast reaches the file tree.

mod helpers;

use helpers::TestHarness;
use txv_core::event::Event;

#[test]
fn roots_changed_broadcast_reaches_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let root_a = tmp.path().to_path_buf();
    let root_b = tmp.path().join("extra");
    std::fs::create_dir_all(&root_b).unwrap();
    std::fs::write(root_b.join("hello.txt"), "hi").unwrap();

    let mut h = TestHarness::new(tmp.path());
    h.run_cycles(1);

    // Do NOT focus tree — center panel (editor) stays focused.
    // Inject CM_ROOTS_CHANGED broadcast
    let data = kairn::commands::RootsChangedData::new(
        vec![root_a, root_b.clone()],
        vec![txv_core::cell::Color::Ansi(2), txv_core::cell::Color::Ansi(3)],
        vec![
            tmp.path().file_name().unwrap().to_string_lossy().to_string(),
            "extra".to_string(),
        ],
    );
    let event = Event::Command {
        id: kairn::commands::CM_ROOTS_CHANGED,
        data: Some(Box::new(data)),
        broadcast: true,
    };
    h.backend.inject(event);
    h.run_cycles(2);

    // Focus tree to see the result
    h.inject_key(txv_core::event::KeyCode::F(2), txv_core::event::KeyMod::default());
    h.run_cycles(1);

    // Tree should now show the extra root name
    let name = root_b.file_name().unwrap().to_string_lossy();
    assert!(
        h.content_contains(&name),
        "tree should show root '{}' after broadcast",
        name
    );
}

#[test]
fn add_root_command_updates_tree() {
    let tmp = tempfile::tempdir().unwrap();
    let extra = tmp.path().join("extra_proj");
    std::fs::create_dir_all(&extra).unwrap();
    std::fs::write(extra.join("file.txt"), "content").unwrap();

    let mut h = TestHarness::new(tmp.path());
    h.run_cycles(1);

    // Execute add-root via dispatch_command (simulates M-x add-root)
    let cmd = format!("add-root {}", extra.display());
    h.dispatch_command(kairn::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
    h.run_cycles(3);

    // Focus tree
    h.inject_key(txv_core::event::KeyCode::F(2), txv_core::event::KeyMod::default());
    h.run_cycles(1);

    let name = extra.file_name().unwrap().to_string_lossy();
    assert!(
        h.content_contains(&name),
        "tree should show root '{}' after add-root command",
        name
    );
}

#[test]
fn root_badges_have_distinct_colors_on_tabs() {
    let tmp = tempfile::tempdir().unwrap();
    let root_a = tmp.path().join("alpha");
    let root_b = tmp.path().join("beta");
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    std::fs::write(root_a.join("a.rs"), "fn a() {}").unwrap();
    std::fs::write(root_b.join("b.rs"), "fn b() {}").unwrap();

    let mut h = TestHarness::new(tmp.path());
    h.run_cycles(1);

    // Add second root
    h.state.roots_mut().add(root_b.clone());
    h.run_cycles(2);

    // Open files from both roots
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(root_a.join("a.rs")))),
    );
    h.run_cycles(2);
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(root_b.join("b.rs")))),
    );
    h.run_cycles(3);

    // Check that root colors are different
    let roots = h.state.roots();
    let color_a = roots.root_for(&root_a.join("a.rs")).color();
    let color_b = roots.root_for(&root_b.join("b.rs")).color();
    assert_ne!(color_a, color_b, "roots should have distinct badge colors");
}
