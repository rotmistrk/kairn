//! Tab-opening handlers: help, messages, shell.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::handler::downcast_desktop;
use crate::handler_evict::try_insert_tab;
use crate::slots::{focus_tab_by_title, next_tab_name, SlotId};
use crate::views::help::HelpView;
use crate::views::messages::MessagesView;
use crate::views::terminal::new_shell_terminal;

pub fn handle_show_help(ctx: &mut CommandContext, state: &mut AppState) {
    let sink = ctx.sink().clone();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        if !focus_tab_by_title(desktop, SlotId::Center, "Help") {
            let help = HelpView::new();
            try_insert_tab(desktop, state, &sink, SlotId::Center, "Help".into(), Box::new(help));
        }
    }
}

pub fn handle_show_messages(ctx: &mut CommandContext, state: &mut AppState) {
    let sink = ctx.sink().clone();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        if focus_tab_by_title(desktop, SlotId::Tools, "Messages") {
            desktop.focus_panel(SlotId::Tools as usize);
        } else {
            let messages = MessagesView::new(state.messages.clone());
            try_insert_tab(
                desktop,
                state,
                &sink,
                SlotId::Tools,
                "Messages".into(),
                Box::new(messages),
            );
            desktop.focus_panel(SlotId::Tools as usize);
        }
    }
}

pub fn handle_new_shell(ctx: &mut CommandContext, state: &mut AppState) {
    let sink = ctx.sink().clone();
    let term = new_shell_terminal();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        let name = next_tab_name(desktop, SlotId::Tools, "Shell");
        try_insert_tab(desktop, state, &sink, SlotId::Tools, name.clone(), term);
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("shell", format!("Started: {name}")))),
        );
    }
}
