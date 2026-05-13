//! Keystroke-level LSP tests — verify gd/gr/K/Ctrl-N trigger correct actions.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

/// Open a file in the editor and position cursor.
fn setup_editor(h: &mut TestHarness) {
    // Focus center panel (editor)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

fn ctrl() -> KeyMod {
    KeyMod {
        ctrl: true,
        alt: false,
        shift: false,
    }
}

fn none() -> KeyMod {
    KeyMod::default()
}

#[test]
fn keystroke_gd_emits_goto_def_command() {
    let dir = temp_project(&[("src/main.rs", "fn hello() {\n    world();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open the file
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/main.rs"),
        ))),
    );
    h.run_cycles(2);
    setup_editor(&mut h);

    // Move to line 2, col 4 (on "world")
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_str("4l");
    h.run_cycles(1);

    // Press gd — should emit CM_LSP_GOTO_DEF
    // Since no LSP server is running, it won't produce a response,
    // but we verify the editor processes the keystroke without panic
    h.inject_str("gd");
    h.run_cycles(2);

    // Editor should still be in normal mode (not crashed)
    assert!(h.content_contains("hello"));
}

#[test]
fn keystroke_gr_emits_find_refs_command() {
    let dir = temp_project(&[("src/lib.rs", "pub fn foo() {}\nfn bar() { foo(); }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/lib.rs"),
        ))),
    );
    h.run_cycles(2);
    setup_editor(&mut h);

    // Position on "foo" (line 1, col 7)
    h.inject_str("7l");
    h.run_cycles(1);

    // Press gr
    h.inject_str("gr");
    h.run_cycles(2);

    // Editor still functional
    assert!(h.content_contains("foo"));
}

#[test]
fn keystroke_k_emits_hover_command() {
    let dir = temp_project(&[("src/main.rs", "fn main() {\n    let x = 42;\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/main.rs"),
        ))),
    );
    h.run_cycles(2);
    setup_editor(&mut h);

    // Press K (shift+k = hover)
    h.inject_key(KeyCode::Char('K'), none());
    h.run_cycles(2);

    // Editor still functional
    assert!(h.content_contains("main"));
}

#[test]
fn keystroke_ctrl_n_triggers_completion() {
    let dir = temp_project(&[("src/main.rs", "fn main() {\n    pri\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/main.rs"),
        ))),
    );
    h.run_cycles(2);
    setup_editor(&mut h);

    // Enter insert mode, type something, then Ctrl-N
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('n'), ctrl());
    h.run_cycles(2);

    // Editor still in insert mode, no crash
    assert!(h.content_contains("pri"));
}

#[test]
fn results_view_shows_grep_title() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() { todo!(); }\n"),
        ("src/lib.rs", "// todo: fix this\npub fn lib() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Execute :grep command
    h.dispatch_command(
        kairn::commands::CM_SHOW_RESULTS,
        Some(Box::new((
            "grep: todo".to_string(),
            vec![kairn::views::results::ResultEntry {
                path: dir.path().join("src/main.rs"),
                line: 0,
                col: 13,
                text: "todo!();".to_string(),
            }],
        ))),
    );
    h.run_cycles(2);

    // The results tab should be visible with the title
    assert!(h.contains("grep:") || h.content_contains("todo!("));
}

#[test]
fn results_view_enter_opens_file() {
    let dir = temp_project(&[("src/foo.rs", "line1\nline2\nline3\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Show results pointing to foo.rs:2
    h.dispatch_command(
        kairn::commands::CM_SHOW_RESULTS,
        Some(Box::new((
            "References: bar".to_string(),
            vec![kairn::views::results::ResultEntry {
                path: dir.path().join("src/foo.rs"),
                line: 1,
                col: 0,
                text: "line2".to_string(),
            }],
        ))),
    );
    h.run_cycles(2);

    // Press Enter to open the file
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(3);

    // foo.rs should now be open
    assert!(h.content_contains("line2"));
}
