//! Build/run/test command handlers.

use txv_core::program::CommandContext;

use crate::build;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;
use crate::views::editor::EditorView;

/// Handle M-x build: run build command, show output in Compile tab.
pub fn handle_build(ctx: &mut CommandContext, state: &mut AppState) {
    let cmd = state
        .settings
        .build_command
        .clone()
        .unwrap_or_else(|| detect_build_command(&state.root_dir));

    let output = build::run_command(&cmd, &state.root_dir).unwrap_or_else(|| "Build failed to start".into());

    let errors = build::parse_errors(&output);
    state.build_errors = errors;
    state.build_error_idx = 0;

    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        desktop.close_tab_by_title(SlotId::Bottom, "Compile");
        let view = EditorView::from_text(&output);
        desktop.insert_tab(SlotId::Bottom, "Compile", Box::new(view));
        desktop.focus_slot(SlotId::Bottom);
    }
}

/// Detect build command from project files.
fn detect_build_command(root: &std::path::Path) -> String {
    if root.join("Cargo.toml").exists() {
        "cargo build".to_string()
    } else if root.join("go.mod").exists() {
        "go build ./...".to_string()
    } else if root.join("package.json").exists() {
        "npm run build".to_string()
    } else {
        "make".to_string()
    }
}

/// Handle M-x run: run the project in a shell tab.
pub fn handle_run(ctx: &mut CommandContext, state: &mut AppState) {
    let cmd = state
        .settings
        .run_command
        .clone()
        .unwrap_or_else(|| detect_run_command(&state.root_dir));

    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        desktop.close_tab_by_title(SlotId::Right, "Run");
        let term = crate::views::terminal::new_shell_with_command(&cmd, &state.root_dir);
        desktop.insert_tab(SlotId::Right, "Run", term);
        desktop.focus_slot(SlotId::Right);
    }
}

/// Detect run command from project files.
fn detect_run_command(root: &std::path::Path) -> String {
    if root.join("Cargo.toml").exists() {
        "cargo run".to_string()
    } else if root.join("go.mod").exists() {
        "go run .".to_string()
    } else if root.join("package.json").exists() {
        "npm start".to_string()
    } else {
        "make run".to_string()
    }
}

/// Handle M-x test: run full test suite, show in Compile tab.
pub fn handle_test(ctx: &mut CommandContext, state: &mut AppState) {
    let cmd = state
        .settings
        .test_command
        .clone()
        .unwrap_or_else(|| detect_test_command(&state.root_dir));
    run_test_command(ctx, state, &cmd);
}

/// Handle M-x test-file: run tests for current file.
pub fn handle_test_file(ctx: &mut CommandContext, state: &mut AppState) {
    let file = state.broker.last_opened().unwrap_or("").to_string();
    let cmd = if state.root_dir.join("Cargo.toml").exists() {
        format!("cargo test --lib {file}")
    } else if state.root_dir.join("go.mod").exists() {
        format!("go test ./{file}")
    } else {
        detect_test_command(&state.root_dir)
    };
    run_test_command(ctx, state, &cmd);
}

/// Handle M-x test-at-cursor: run test under cursor.
pub fn handle_test_at_cursor(ctx: &mut CommandContext, state: &mut AppState) {
    // Try to detect test function name from cursor context
    let cmd = if state.root_dir.join("Cargo.toml").exists() {
        "cargo test".to_string()
    } else {
        detect_test_command(&state.root_dir)
    };
    run_test_command(ctx, state, &cmd);
}

fn run_test_command(ctx: &mut CommandContext, state: &mut AppState, cmd: &str) {
    let output = build::run_command(cmd, &state.root_dir).unwrap_or_else(|| "Test failed to start".into());
    let errors = build::parse_errors(&output);
    state.build_errors = errors;
    state.build_error_idx = 0;

    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        desktop.close_tab_by_title(SlotId::Bottom, "Compile");
        let view = EditorView::from_text(&output);
        desktop.insert_tab(SlotId::Bottom, "Compile", Box::new(view));
        desktop.focus_slot(SlotId::Bottom);
    }
}

fn detect_test_command(root: &std::path::Path) -> String {
    if root.join("Cargo.toml").exists() {
        "cargo test".to_string()
    } else if root.join("go.mod").exists() {
        "go test ./...".to_string()
    } else if root.join("package.json").exists() {
        "npm test".to_string()
    } else {
        "make test".to_string()
    }
}

/// Handle M-x next-error: jump to next error location.
pub fn handle_next_error(ctx: &mut CommandContext, state: &mut AppState) {
    if state.build_errors.is_empty() {
        return;
    }
    if state.build_error_idx < state.build_errors.len() - 1 {
        state.build_error_idx += 1;
    }
    jump_to_error(ctx, state);
}

/// Handle M-x prev-error: jump to previous error location.
pub fn handle_prev_error(ctx: &mut CommandContext, state: &mut AppState) {
    if state.build_errors.is_empty() {
        return;
    }
    if state.build_error_idx > 0 {
        state.build_error_idx -= 1;
    }
    jump_to_error(ctx, state);
}

fn jump_to_error(ctx: &mut CommandContext, state: &mut AppState) {
    let err = &state.build_errors[state.build_error_idx];
    let path = state.root_dir.join(&err.file);
    let req = crate::commands::OpenFileRequest::at(path, err.line.saturating_sub(1), err.col.saturating_sub(1));
    ctx.queue
        .put_command(crate::commands::CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
}
