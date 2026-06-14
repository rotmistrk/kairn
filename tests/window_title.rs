//! Tests for window title expression evaluation.

mod helpers;

use helpers::TestHarness;
use kairn::scripting::ScriptEngine;

// --- Unit tests for ScriptEngine::subst ---

#[test]
fn subst_plain_text() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.subst("hello world").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn subst_command_substitution() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("set x hello").unwrap();
    let result = engine.subst("$x world").unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn subst_system_user() {
    let mut engine = ScriptEngine::new(None);
    let user = std::env::var("USER").unwrap_or_default();
    let result = engine.subst("kairn:[system user]").unwrap();
    assert_eq!(result, format!("kairn:{user}"));
}

#[test]
fn subst_multiple_commands() {
    let mut engine = ScriptEngine::new(None);
    let user = std::env::var("USER").unwrap_or_default();
    let result = engine.subst("[system user]@[system hostname 0]").unwrap();
    assert!(result.starts_with(&format!("{user}@")), "got: {result}");
}

#[test]
fn subst_preserves_special_chars() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.subst("path/to/file.rs").unwrap();
    assert_eq!(result, "path/to/file.rs");
}

#[test]
fn subst_error_reports_message() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.subst("[system bad_subcommand]");
    assert!(result.is_err(), "expected error, got: {:?}", result);
}

// --- Scenario test: window title updates via context broadcast ---

#[test]
fn window_title_updates_on_context() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = TestHarness::new(dir.path());

    // Set a simple title expression
    std::env::set_var("USER", "tester");
    h.state
        .script_mut()
        .eval("set window.title-expr {kairn:[system user]}")
        .unwrap();

    // Trigger context update (which calls update_window_title)
    h.dispatch_command(txv_core::commands::CM_TICK, None);
    h.run_cycles(3);

    assert_eq!(h.state.last_window_title(), "kairn:tester");
}
