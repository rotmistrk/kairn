//! Misc M-x command handlers (theme, vsplit, welcome, set).

use txv_core::program::CommandContext;

use crate::commands::*;
use crate::desktop::{focus_tab_by_title, SlotId};
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::views::welcome::WelcomeView;

pub(crate) fn cmd_theme(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    if let Some(name) = arg.strip_prefix("syntax ") {
        ctx.sink()
            .push_command(CM_SET_SYNTAX_THEME, Some(Box::new(name.to_string())));
    } else if let Some(g) = arg.strip_prefix("glyphs ") {
        ctx.sink().push_command(CM_SET_GLYPHS, Some(Box::new(g.to_string())));
    } else if matches!(arg, "dark" | "light" | "auto" | "toggle" | "") {
        ctx.sink()
            .push_command(CM_TOGGLE_THEME, Some(Box::new(arg.to_string())));
    }
}

pub(crate) fn cmd_vsplit(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let req = SplitRequest {
        vertical: true,
        file: if arg.is_empty() {
            None
        } else {
            Some(arg.to_string())
        },
    };
    ctx.sink().push_command(CM_SPLIT, Some(Box::new(req)));
}

pub(crate) fn cmd_welcome(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        if !focus_tab_by_title(desktop, SlotId::Center, "Welcome") {
            try_insert_tab(
                desktop,
                state,
                &sink,
                SlotId::Center,
                "Welcome".into(),
                Box::new(WelcomeView::new(state.root_dir.clone())),
            );
        }
    }
}

pub(crate) fn cmd_set(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_SET_GLOBAL, Some(Box::new(arg.to_string())));
}
