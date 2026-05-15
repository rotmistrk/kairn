//! Tests for the Tcl scripting bridge.

use kairn::scripting::{ScriptCommand, ScriptEngine};

#[test]
fn editor_open_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("editor open test.rs").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::OpenFile { path, .. } if path == "test.rs"));
}

#[test]
fn editor_open_with_line_flag() {
    let mut engine = ScriptEngine::new();
    engine.eval("editor open main.rs -line 42").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::OpenFile { path, line: Some(42), .. } if path == "main.rs"));
}

#[test]
fn editor_save_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("editor save").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::Save));
}

#[test]
fn editor_goto_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("editor goto 10 5").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::Goto { line: 10, col: 5 }));
}

#[test]
fn editor_query_returns_snapshot_data() {
    let mut engine = ScriptEngine::new();
    let ctx = kairn::commands::ViewContext {
        file: Some("src/main.rs".into()),
        line: 42,
        col: 7,
        modified: true,
        language: "rust".into(),
        ..Default::default()
    };
    engine.update_snapshot(&ctx, "/project", "", "");
    assert_eq!(engine.eval("editor current-file").unwrap(), "src/main.rs");
    assert_eq!(engine.eval("editor current-line").unwrap(), "42");
    assert_eq!(engine.eval("editor current-col").unwrap(), "7");
    assert_eq!(engine.eval("editor modified?").unwrap(), "1");
    assert_eq!(engine.eval("editor filetype").unwrap(), "rust");
}

#[test]
fn view_message_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("view message error tcl {something broke}").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ShowMessage { level, origin, text }
        if level == "error" && origin == "tcl" && text == "something broke"));
}

#[test]
fn view_focus_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("view focus left").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::FocusSlot { slot } if slot == "left"));
}

#[test]
fn system_platform_returns_value() {
    let mut engine = ScriptEngine::new();
    let result = engine.eval("system platform").unwrap();
    assert!(result == "linux" || result == "macos");
}

#[test]
fn system_root_dir_returns_snapshot() {
    let mut engine = ScriptEngine::new();
    let ctx = kairn::commands::ViewContext::default();
    engine.update_snapshot(&ctx, "/home/user/project", "", "");
    assert_eq!(engine.eval("system root-dir").unwrap(), "/home/user/project");
}

#[test]
fn build_run_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("build run").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::RunBuild { command: None }));
}

#[test]
fn build_test_with_arg() {
    let mut engine = ScriptEngine::new();
    engine.eval("build test {cargo test --lib}").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::RunTest { command: Some(c) } if c == "cargo test --lib"));
}

#[test]
fn keymap_bind_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("keymap bind ctrl+s save").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::SetKeyBinding { key, command }
        if key == "ctrl+s" && command == "save"));
}

#[test]
fn hook_add_and_list() {
    let mut engine = ScriptEngine::new();
    engine.eval("hook add file-save {puts saved}").unwrap();
    let result = engine.eval("hook list file-save").unwrap();
    assert!(result.contains("saved"));
}

#[test]
fn lsp_rename_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("lsp rename new_name").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::LspRename { new_name } if new_name == "new_name"));
}

#[test]
fn git_commit_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("git commit {fix bug}").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::GitCommit { message } if message == "fix bug"));
}

#[test]
fn todo_add_produces_command() {
    let mut engine = ScriptEngine::new();
    engine.eval("todo add {write tests}").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoAdd { text, parent: None } if text == "write tests"));
}

#[test]
fn unknown_subcommand_returns_error() {
    let mut engine = ScriptEngine::new();
    let result = engine.eval("editor nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unknown subcommand"));
}

#[test]
fn tcl_script_with_variables() {
    let mut engine = ScriptEngine::new();
    engine.eval("set x hello").unwrap();
    let result = engine.eval("set x").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn multiple_commands_in_script() {
    let mut engine = ScriptEngine::new();
    engine.eval("editor save\neditor close").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 2);
    assert!(matches!(&cmds[0], ScriptCommand::Save));
    assert!(matches!(&cmds[1], ScriptCommand::Close));
}
