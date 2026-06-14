//! Tests for Tcl scripting bridge commands (build, git, view, grep, todo).

use kairn::scripting::{ScriptCommand, ScriptEngine};

// ─── build namespace ────────────────────────────────────────────────────────

#[test]
fn build_test_file_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("build test-file").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TestFile));
}

#[test]
fn build_test_at_cursor_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("build test-at-cursor").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TestAtCursor));
}

#[test]
fn build_next_error_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("build next-error").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::NextError));
}

#[test]
fn build_prev_error_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("build prev-error").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::PrevError));
}

// ─── git namespace ──────────────────────────────────────────────────────────

#[test]
fn git_untrack_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("git untrack src/old.rs").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::GitUntrack { file } if file == "src/old.rs"));
}

#[test]
fn git_log_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("git log").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::GitLog));
}

#[test]
fn git_diff_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("git diff").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::GitDiff));
}

#[test]
fn git_blame_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("git blame").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::GitBlame));
}

#[test]
fn git_noblame_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("git noblame").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::GitNoBlame));
}

// ─── view namespace ─────────────────────────────────────────────────────────

#[test]
fn view_theme_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("view theme dark").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ViewTheme { mode } if mode == "dark"));
}

#[test]
fn view_theme_toggle() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("view theme toggle").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ViewTheme { mode } if mode == "toggle"));
}

#[test]
fn view_zoom_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("view zoom").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ViewZoom));
}

#[test]
fn view_toggle_tree_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("view toggle-tree").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ViewToggleTree));
}

#[test]
fn view_toggle_tools_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("view toggle-tools").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ViewToggleTools));
}

#[test]
fn view_layout_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("view layout").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::ViewLayout));
}

// ─── grep ───────────────────────────────────────────────────────────────────

#[test]
fn grep_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("grep TODO").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::Grep { pattern } if pattern == "TODO"));
}

#[test]
fn grep_with_spaces() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("grep {fn main}").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::Grep { pattern } if pattern == "fn main"));
}

// ─── todo namespace ─────────────────────────────────────────────────────────

#[test]
fn todo_add_with_parent() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo add {subtask} -parent 0.1").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoAdd { text, parent: Some(p) }
        if text == "subtask" && p == "0.1"));
}

#[test]
fn todo_toggle_important_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo toggle-important 0").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoToggleImportant { path } if path == "0"));
}

#[test]
fn todo_edit_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo edit 0.1 {new title}").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoEdit { path, text }
        if path == "0.1" && text == "new title"));
}

#[test]
fn todo_swap_up_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo swap 2 up").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoSwap { path, direction }
        if path == "2" && direction == "up"));
}

#[test]
fn todo_swap_down_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo swap 0 down").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoSwap { path, direction }
        if path == "0" && direction == "down"));
}

#[test]
fn todo_promote_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo promote 1.0").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoPromote { path } if path == "1.0"));
}

#[test]
fn todo_demote_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo demote 1").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoDemote { path } if path == "1"));
}

#[test]
fn todo_list_produces_command() {
    let mut engine = ScriptEngine::new(None);
    engine.eval("todo list").unwrap();
    let cmds = engine.drain_commands();
    assert_eq!(cmds.len(), 1);
    assert!(matches!(&cmds[0], ScriptCommand::TodoList));
}
