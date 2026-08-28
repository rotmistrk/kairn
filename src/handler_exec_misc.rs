//! Misc M-x command handlers (theme, vsplit, welcome, set).

use std::fs;
use std::path::PathBuf;

use txv_core::message::Message;
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
                Box::new(WelcomeView::new(state.root_dir().clone())),
            );
        }
    }
}

pub(crate) fn cmd_set(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_SET_GLOBAL, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_add_root(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let path = PathBuf::from(arg);
    let path = if path.is_relative() {
        state.root_dir().join(&path)
    } else {
        path
    };
    let Some(path) = fs::canonicalize(&path).ok() else {
        push_msg(state, Message::error("root", format!("Not found: {arg}")));
        return;
    };
    if !path.is_dir() {
        push_msg(state, Message::error("root", format!("Not a directory: {arg}")));
        return;
    }
    if !state.roots_mut().add(path.clone()) {
        push_msg(
            state,
            Message::warn("root", format!("Already a root: {}", path.display())),
        );
        return;
    }
    push_msg(state, Message::info("root", format!("Added root: {}", path.display())));
    refresh_completer_roots(state);
    emit_roots_changed(ctx, state);
}
pub(crate) fn cmd_remove_root(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let path = PathBuf::from(arg);
    let path = if path.is_relative() {
        state.root_dir().join(&path)
    } else {
        path
    };
    let path = fs::canonicalize(&path).unwrap_or(path);
    if !state.roots_mut().remove(&path) {
        push_msg(
            state,
            Message::warn("root", format!("Not a root or last root: {}", path.display())),
        );
        return;
    }
    push_msg(
        state,
        Message::info("root", format!("Removed root: {}", path.display())),
    );
    refresh_completer_roots(state);
    emit_roots_changed(ctx, state);
}
pub(crate) fn refresh_completer_roots(state: &AppState) {
    let paths: Vec<String> = state.roots().paths().iter().map(|p| p.display().to_string()).collect();
    if let Ok(mut guard) = state.scripting().completer_roots().lock() {
        *guard = paths;
    }
}
fn emit_roots_changed(ctx: &mut CommandContext, state: &AppState) {
    let data = RootsChangedData::from_roots(state.roots());
    ctx.sink().push_broadcast(CM_ROOTS_CHANGED, Some(Box::new(data)));
}
fn push_msg(state: &AppState, msg: Message) {
    if let Ok(mut ring) = state.messages().lock() {
        ring.push(msg);
    }
}
