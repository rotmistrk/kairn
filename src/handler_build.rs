//! Build/run/test command handlers.

use txv_core::program::CommandContext;

use crate::build;
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
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
