//! Test: Tcl eval via M-x dispatches ScriptCommands (editor open, goto).

mod helpers;

use helpers::TestHarness;

#[test]
fn tcl_editor_open_via_mx() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("hello.txt");
    std::fs::write(&file, "line 1\nline 2\nline 3\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    // Execute: editor open <file>
    let cmd = format!("editor open {}", file.display());
    h.dispatch_command(kairn::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
    h.run_cycles(5);

    // File should be open in center
    assert!(h.content_contains("hello.txt"), "tab title should show hello.txt");
}

#[test]
fn tcl_goto_moves_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("lines.txt");
    let content: String = (1..=20).map(|i| format!("line {i}\n")).collect();
    std::fs::write(&file, &content).unwrap();

    let mut h = TestHarness::new(dir.path());
    // Open file first
    let open_cmd = format!("editor open {}", file.display());
    h.dispatch_command(kairn::commands::CM_EXECUTE_COMMAND, Some(Box::new(open_cmd)));
    h.run_cycles(3);

    // Now goto line 10
    h.dispatch_command(
        kairn::commands::CM_EXECUTE_COMMAND,
        Some(Box::new("editor goto 10".to_string())),
    );
    h.run_cycles(3);

    // Line 10 should be visible
    assert!(h.content_contains("line 10"), "line 10 should be visible after goto");
}
