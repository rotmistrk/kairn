//! Scenario tests for build/test/run Tcl override mechanism.

mod helpers;

use helpers::{temp_project, TestHarness};

#[test]
fn build_command_override_via_tcl_proc() {
    let dir = temp_project(&[
        ("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ("src/main.rs", "fn main() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());

    // Define override proc
    h.state
        .script
        .eval("proc build-command {} { return {echo custom-build} }")
        .unwrap();

    // Verify the proc is callable and returns the override
    assert!(h.state.script.has_command("build-command"));
    let result = h.state.script.eval("build-command").unwrap();
    assert_eq!(result, "echo custom-build");
}

#[test]
fn test_command_override_via_tcl_proc() {
    let dir = temp_project(&[
        ("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ("src/main.rs", "fn main() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());

    h.state
        .script
        .eval("proc test-command {} { return {echo custom-test} }")
        .unwrap();
    let result = h.state.script.eval("test-command").unwrap();
    assert_eq!(result, "echo custom-test");
}

#[test]
fn run_command_override_via_tcl_proc() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());

    h.state
        .script
        .eval("proc run-command {} { return {./my-app --debug} }")
        .unwrap();
    let result = h.state.script.eval("run-command").unwrap();
    assert_eq!(result, "./my-app --debug");
}

#[test]
fn build_command_empty_return_falls_through() {
    let dir = temp_project(&[
        ("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ("src/main.rs", "fn main() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());

    // Empty return means "use default"
    h.state.script.eval("proc build-command {} { return {} }").unwrap();
    let result = h.state.script.eval("build-command").unwrap();
    assert_eq!(result, "");
}

#[test]
fn build_command_override_can_use_editor_context() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());

    // Override that uses editor context
    h.state
        .script
        .eval(
            r#"
        proc build-command {} {
            set file [editor current-file]
            return "echo building $file"
        }
    "#,
        )
        .unwrap();

    // With no file open, returns "echo building "
    let ctx = kairn::commands::ViewContext::default();
    h.state
        .script
        .update_snapshot(&ctx, dir.path().to_str().unwrap(), "", "", "none", false);
    let result = h.state.script.eval("build-command").unwrap();
    assert_eq!(result, "echo building ");

    // With a file open, returns custom command with file name
    let ctx = kairn::commands::ViewContext {
        file: Some("src/main.rs".into()),
        ..Default::default()
    };
    h.state
        .script
        .update_snapshot(&ctx, dir.path().to_str().unwrap(), "", "", "none", false);
    let result = h.state.script.eval("build-command").unwrap();
    assert_eq!(result, "echo building src/main.rs");
}

#[test]
fn project_init_tcl_overrides_global() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() {}\n"),
        (".kairn/init.tcl", "proc build-command {} { return {make -j4} }"),
    ]);
    let mut h = TestHarness::new(dir.path());

    // Load project config (simulates what the app does on startup)
    h.state.script.load_config(dir.path());

    assert!(h.state.script.has_command("build-command"));
    let result = h.state.script.eval("build-command").unwrap();
    assert_eq!(result, "make -j4");
}
