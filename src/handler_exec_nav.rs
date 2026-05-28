//! Navigation/layout handler functions for M-x dispatch.

use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::commands::{
    CM_TW_CYCLE_SUBPANEL, CM_TW_FOCUS_DOWN, CM_TW_FOCUS_LEFT, CM_TW_FOCUS_RIGHT, CM_TW_FOCUS_UP, CM_TW_GROW_H,
    CM_TW_GROW_SUBPANEL, CM_TW_GROW_V, CM_TW_LAYOUT_CYCLE, CM_TW_MOVE_TAB_SUBPANEL, CM_TW_SHRINK_H,
    CM_TW_SHRINK_SUBPANEL, CM_TW_SHRINK_V, CM_TW_TAB_NEXT, CM_TW_TAB_PREV, CM_TW_TOGGLE_TOOLS, CM_TW_TOGGLE_TREE,
    CM_TW_ZOOM,
};

use crate::handler::{downcast_desktop, AppState};

pub(crate) fn cmd_cycle_subpanel(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_CYCLE_SUBPANEL, None);
}

pub(crate) fn cmd_focus_down(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_FOCUS_DOWN, None);
}

pub(crate) fn cmd_focus_left(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_FOCUS_LEFT, None);
}

pub(crate) fn cmd_focus_right(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_FOCUS_RIGHT, None);
}

pub(crate) fn cmd_focus_up(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_FOCUS_UP, None);
}

pub(crate) fn cmd_grow(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_GROW_H, None);
}

pub(crate) fn cmd_grow_subpanel(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_GROW_SUBPANEL, None);
}

pub(crate) fn cmd_grow_v(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_GROW_V, None);
}

pub(crate) fn cmd_layout(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_LAYOUT_CYCLE, None);
}

pub(crate) fn cmd_move_tab(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_MOVE_TAB_SUBPANEL, None);
}

pub(crate) fn cmd_shrink(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_SHRINK_H, None);
}

pub(crate) fn cmd_shrink_subpanel(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_SHRINK_SUBPANEL, None);
}

pub(crate) fn cmd_shrink_v(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_SHRINK_V, None);
}

pub(crate) fn cmd_tab_next(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_TAB_NEXT, None);
}

pub(crate) fn cmd_tab_prev(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_TAB_PREV, None);
}

pub(crate) fn cmd_toggle_tools(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_TOGGLE_TOOLS, None);
}

pub(crate) fn cmd_toggle_tree(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_TOGGLE_TREE, None);
}

pub(crate) fn cmd_zoom(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink.push_command(CM_TW_ZOOM, None);
}

pub(crate) fn cmd_tab_rename(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let slot = desktop.focused_panel();
    let old_title = desktop.panel(slot).and_then(|p| p.active_title().map(String::from));
    if let Some(panel) = desktop.panel_mut(slot) {
        panel.rename_user_part(arg);
    }
    let Some(old) = old_title else {
        return;
    };
    if !state.kiro_registry.contains(&old) {
        return;
    }
    let new_title = desktop.panel(slot).and_then(|p| p.active_title().map(String::from));
    if let Some(new) = new_title {
        state.kiro_registry.rename(&old, &new);
    }
}
