//! Handler for :replace / :sed command — project-wide search and replace.

use std::thread;
use std::time::Duration;

use txv_core::message::Message;
use txv_core::program::CommandContext;
use txv_core::run::Waker;

use crate::desktop::SlotId;
use crate::grep::grep_async;
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::handler_exec_edit::push_status;
use crate::views::search_replace::SearchReplaceView;

pub(crate) fn cmd_replace(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let sink = ctx.sink().clone();
    let Some((pattern, replacement)) = parse_replace_arg(arg) else {
        push_status(ctx, Message::warn("replace", "Usage: :replace /pattern/replacement/"));
        return;
    };
    let root = state.root_dir.clone();
    let waker = state.waker.clone().unwrap_or_else(Waker::noop);
    let grep_state = grep_async(&pattern, &root, waker);
    thread::sleep(Duration::from_millis(500));
    let entries = grep_state.take_entries();
    if entries.is_empty() {
        push_status(ctx, Message::info("replace", "No matches found"));
        return;
    }
    let view = SearchReplaceView::new(&pattern, &replacement, &root, entries);
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        let title = format!("replace:{pattern}");
        try_insert_tab(desktop, state, &sink, SlotId::Tools, title, Box::new(view));
        desktop.focus_panel(SlotId::Tools as usize);
    }
}

fn parse_replace_arg(arg: &str) -> Option<(String, String)> {
    let arg = arg.trim();
    if let Some(rest) = arg.strip_prefix('/') {
        let sep = rest.find('/')?;
        let pattern = rest[..sep].to_string();
        let after = &rest[sep + 1..];
        let replacement = after.trim_end_matches('/').to_string();
        Some((pattern, replacement))
    } else {
        let mut parts = arg.splitn(2, ' ');
        let pattern = parts.next()?.to_string();
        let replacement = parts.next().unwrap_or("").to_string();
        if pattern.is_empty() {
            None
        } else {
            Some((pattern, replacement))
        }
    }
}
