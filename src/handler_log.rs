//! Handler for M-x log — opens git commit log in the right panel.

use txv_core::program::CommandContext;

use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;

/// Open the git log viewer as a singleton tab in the right panel.
pub fn open_git_log(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    desktop.close_tab_by_title(SlotId::Right, "Log");

    let filter_path = if arg == "%" {
        desktop
            .active_view_mut(SlotId::Center)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_ref::<crate::views::editor::EditorView>())
            .map(|e| e.path().strip_prefix(&state.root_dir).unwrap_or(e.path()).to_path_buf())
    } else {
        None
    };
    let branch = if arg == "%" || arg.is_empty() {
        None
    } else {
        Some(arg)
    };

    let shared = crate::git_log::log_async(&state.root_dir, branch, filter_path.as_deref());
    let view = crate::views::git_log::GitLogView::new(shared);
    crate::handler_evict::try_insert_tab(
        desktop,
        state,
        ctx.sink,
        SlotId::Right,
        "Log".to_string(),
        Box::new(view),
    );
    desktop.focus_slot(SlotId::Right);
}
