//! Build/run/test command handlers — async execution with ResultsView output.

use txv_core::program::CommandContext;

use crate::build;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;

/// Handle :build — spawn async build, show results in right panel.
pub fn handle_build(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(cmd) = build::resolve_build_cmd(&state.root_dir) else {
        report_no_cmd(ctx, "build");
        return;
    };
    spawn_task(ctx, state, &cmd, "Build");
}

/// Handle :run — run project in a shell tab.
pub fn handle_run(ctx: &mut CommandContext, state: &mut AppState) {
    let cmd = state
        .settings
        .run_command
        .clone()
        .unwrap_or_else(|| detect_run_command(&state.root_dir));
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        desktop.close_tab_by_title(SlotId::Right, "Run");
        let term = crate::views::terminal::new_shell_with_command(&cmd, &state.root_dir);
        crate::handler_evict::try_insert_tab(desktop, state, ctx.queue, SlotId::Right, "Run".into(), term);
        desktop.focus_slot(SlotId::Right);
    }
}

/// Handle :test — spawn async test, show results.
pub fn handle_test(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(cmd) = build::resolve_test_cmd(&state.root_dir) else {
        report_no_cmd(ctx, "test");
        return;
    };
    spawn_task(ctx, state, &cmd, "Test");
}

/// Handle :test-file — test current file.
pub fn handle_test_file(ctx: &mut CommandContext, state: &mut AppState) {
    let file = state.broker.last_opened().unwrap_or("").to_string();
    let Some(cmd) = build::resolve_test_file_cmd(&state.root_dir, &file) else {
        report_no_cmd(ctx, "test-file");
        return;
    };
    spawn_task(ctx, state, &cmd, "Test");
}

/// Handle :test-at-cursor — test function at cursor.
pub fn handle_test_at_cursor(ctx: &mut CommandContext, state: &mut AppState) {
    let test_name = detect_test_name(ctx, state);
    let Some(cmd) = build::resolve_test_at_cursor_cmd(&state.root_dir, &test_name) else {
        report_no_cmd(ctx, "test-at-cursor");
        return;
    };
    spawn_task(ctx, state, &cmd, "Test");
}

/// Spawn an async build/test task and open a ResultsView.
fn spawn_task(ctx: &mut CommandContext, state: &mut AppState, cmd: &str, title: &str) {
    let Some(waker) = state.waker.clone() else {
        return;
    };
    let root = state.root_dir.clone();
    let task = build::run_async(cmd, &root, waker);
    state.build_pending = Some((title.to_string(), task, root));
    state.build_errors.clear();
    state.build_error_idx = 0;

    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        desktop.close_tab_by_title(SlotId::Right, title);
        let view = crate::views::results::ResultsView::searching(title, &state.root_dir);
        crate::handler_evict::try_insert_tab(
            desktop,
            state,
            ctx.queue,
            SlotId::Right,
            title.to_string(),
            Box::new(view),
        );
        desktop.focus_slot(SlotId::Right);
    }
}

fn report_no_cmd(ctx: &mut CommandContext, what: &str) {
    use txv_core::message::Message;
    let msg = Message::error("build", format!("No {what} command configured"));
    ctx.queue
        .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
}

fn detect_run_command(root: &std::path::Path) -> String {
    if root.join("Cargo.toml").exists() {
        "cargo run".to_string()
    } else if root.join("go.mod").exists() {
        "go run .".to_string()
    } else {
        "make run".to_string()
    }
}

/// Try to detect the test function name at the cursor position.
fn detect_test_name(ctx: &mut CommandContext, _state: &AppState) -> String {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        if let Some(view) = desktop.active_view_mut(SlotId::Center) {
            if let Some(any) = view.as_any_mut() {
                if let Some(editor) = any.downcast_mut::<crate::views::editor::EditorView>() {
                    let line = editor.editor.cursor_line;
                    // Walk backwards from cursor to find fn name
                    for i in (0..=line).rev() {
                        let text = editor.editor.buf().line(i).unwrap_or_default();
                        if let Some(name) = extract_test_fn_name(&text) {
                            return name;
                        }
                    }
                }
            }
        }
    }
    String::new()
}

/// Extract test function name from a line like `fn test_foo()` or `#[test]`.
fn extract_test_fn_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // Match "fn name(" or "pub fn name(" etc.
    if let Some(idx) = trimmed.find("fn ") {
        let rest = &trimmed[idx + 3..];
        let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// Handle :next-error — jump to next error location.
pub fn handle_next_error(ctx: &mut CommandContext, state: &mut AppState) {
    if state.build_errors.is_empty() {
        return;
    }
    if state.build_error_idx < state.build_errors.len() - 1 {
        state.build_error_idx += 1;
    }
    jump_to_error(ctx, state);
}

/// Handle :prev-error — jump to previous error location.
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
