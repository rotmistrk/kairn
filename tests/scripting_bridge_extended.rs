//! Tests for Tcl scripting bridge commands (editor, lsp, split, errors, procs).

use kairn::scripting::{ScriptCommand, ScriptEngine};

// ─── editor search/clear-highlight ──────────────────────────────────────────

#[test]
fn editor_search_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("editor search pattern").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::Search { pattern: Some(p) } if p == "pattern"));
}

#[test]
fn editor_clear_highlight_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("editor clear-highlight").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::Search { pattern: None }));
}

// ─── lsp lifecycle ──────────────────────────────────────────────────────────

#[test]
fn lsp_start_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("lsp start rust-analyzer").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::LspStart { pattern } if pattern == "rust-analyzer"));
}

#[test]
fn lsp_restart_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("lsp restart *").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::LspRestart { pattern } if pattern == "*"));
}

#[test]
fn lsp_stop_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("lsp stop typescript").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::LspStop { pattern } if pattern == "typescript"));
}

#[test]
fn lsp_timeout_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("lsp timeout rust-analyzer 30").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::LspTimeout { pattern, secs: Some(30) }
        if pattern == "rust-analyzer"));
}

#[test]
fn lsp_args_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine
        .eval("lsp args rust-analyzer {rust-analyzer --log-file /tmp/ra.log}")
        .unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::LspArgs { pattern, command }
        if pattern == "rust-analyzer" && command == "rust-analyzer --log-file /tmp/ra.log"));
}

// ─── split namespace ────────────────────────────────────────────────────────

#[test]
fn split_vsplit_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split vsplit").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitVertical { file: None }));
}

#[test]
fn split_vsplit_with_file() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split vsplit src/main.rs").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitVertical { file: Some(f) } if f == "src/main.rs"));
}

#[test]
fn split_hsplit_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split hsplit").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitHorizontal { file: None }));
}

#[test]
fn split_close_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split close").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitClose));
}

#[test]
fn split_focus_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split focus").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitFocus));
}

#[test]
fn split_open_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split open lib.rs").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitOpen { path } if path == "lib.rs"));
}

#[test]
fn split_linked_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("split linked true").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SplitLinked { on: true }));
}

// ─── error cases ────────────────────────────────────────────────────────────

#[test]
fn view_unknown_subcommand_errors() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.eval("view nonexistent");
    assert!(result.is_err());
}

#[test]
fn git_unknown_subcommand_errors() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.eval("git nonexistent");
    assert!(result.is_err());
}

#[test]
fn build_unknown_subcommand_errors() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.eval("build nonexistent");
    assert!(result.is_err());
}

#[test]
fn todo_unknown_subcommand_errors() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.eval("todo nonexistent");
    assert!(result.is_err());
}

#[test]
fn split_unknown_subcommand_errors() {
    let mut engine = ScriptEngine::new(None);
    let result = engine.eval("split nonexistent");
    assert!(result.is_err());
}

// ─── build-command override proc ────────────────────────────────────────────

#[test]
fn build_command_proc_override() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("proc build-command {} { return {make -j8} }").unwrap();
    assert!(engine.has_command("build-command"));
    let result = engine.eval("build-command").unwrap();
    assert_eq!(result, "make -j8");
}

#[test]
fn test_command_proc_override() {
    let mut engine = ScriptEngine::new(None);
    engine
        .eval("proc test-command {} { return {cargo test --workspace} }")
        .unwrap();
    let result = engine.eval("test-command").unwrap();
    assert_eq!(result, "cargo test --workspace");
}

#[test]
fn run_command_proc_override() {
    let mut engine = ScriptEngine::new(None);
    engine
        .eval("proc run-command {} { return {./target/debug/myapp} }")
        .unwrap();
    let result = engine.eval("run-command").unwrap();
    assert_eq!(result, "./target/debug/myapp");
}

#[test]
fn build_command_proc_empty_falls_through() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("proc build-command {} { return {} }").unwrap();
    let result = engine.eval("build-command").unwrap();
    assert_eq!(result, "");
}
