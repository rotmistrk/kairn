//! Tests for DiffView command line — history and completion.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

/// Create a git project with initial commit, then modify the file.
fn git_project_modified(filename: &str, initial: &str, modified: &str) -> tempfile::TempDir {
    let dir = temp_project(&[(filename, initial)]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    std::fs::write(dir.path().join(filename), modified).unwrap();
    dir
}

fn open_file(h: &mut TestHarness, name: &str) {
    let path = h.state.root_dir().join(name);
    let req = OpenFileRequest::new(path);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);
}

fn send_ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    for ch in cmd.chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(5);
}

/// Bug 2: DiffView command line should preserve history across activations.
/// When user types :q then activates : again and presses Up, they should see "q".
#[test]
fn diff_cmdline_has_history() {
    let dir = git_project_modified("hist.rs", "old\n", "new\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "hist.rs");
    send_ex(&mut h, "diff");

    // Type a command that doesn't exit (e.g., nonsense that gets ignored)
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    for ch in "noop".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    // Activate cmdline again and press Up — should show previous command
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Up, KeyMod::default());
    h.run_cycles(1);

    let text = h.screen_text();
    assert!(
        text.contains("noop"),
        "History should show previous command 'noop'. Screen:\n{text}"
    );
}

/// Bug 5: Tab completion in DiffView cmdline should suggest diff options.
/// After typing "v" and pressing Tab, the single match "vdiff" should be inserted.
#[test]
fn diff_cmdline_tab_completes_options() {
    let dir = git_project_modified("compl.rs", "old\n", "new\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "compl.rs");
    send_ex(&mut h, "diff");

    // Activate cmdline and type "v" then press Tab — should complete to "vdiff"
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_key(KeyCode::Char('v'), KeyMod::default());
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(3);

    // After single-match Tab completion, "vdiff" should appear on screen
    let screen = h.screen_text();
    assert!(
        screen.contains("vdiff"),
        "Tab should complete 'v' to 'vdiff'. Screen:\n{screen}"
    );
}
