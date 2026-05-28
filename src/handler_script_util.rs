//! Script utility functions: hook firing, slot lookup, LSP command dispatch.

use txv_core::program::CommandContext;
use txv_widgets::CM_STATUS_MESSAGE;

use crate::app_state::AppState;
use crate::desktop::SlotId;

use crate::handler_script::dispatch_script_commands;

/// Slot name to SlotId conversion.
pub fn slot_from_name(name: &str) -> Option<SlotId> {
    match name {
        "left" => Some(SlotId::Left),
        "center" => Some(SlotId::Center),
        "right" => Some(SlotId::Tools),
        _ => None,
    }
}

/// Fire hooks for an event, eval resulting scripts, and dispatch their commands.
pub fn fire_hooks_for_event(
    state: &mut AppState,
    event: &crate::scripting::hooks::HookEvent,
    context: &str,
    ctx: &mut CommandContext,
) {
    let scripts = if let Ok(reg) = state.script.hook_registry.lock() {
        reg.fire(event, context)
    } else {
        return;
    };
    for script in scripts {
        if let Ok(_result) = state.script.eval(&script) {
            let cmds = state.script.drain_commands();
            dispatch_script_commands(cmds, ctx, state);
        }
    }
}

pub fn lsp_cmd(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let msg = crate::handler_lsp_cmd::handle_lsp_command(arg, state);
    ctx.sink.push_command(
        CM_STATUS_MESSAGE,
        Some(Box::new(txv_core::message::Message::info("lsp", msg))),
    );
}

/// Fire lsp-start hook for a language. Runs synchronously (no CommandContext needed).
pub fn fire_lsp_start_hook(state: &mut AppState, language_id: &str) {
    let scripts = if let Ok(reg) = state.script.hook_registry.lock() {
        reg.fire(&crate::scripting::hooks::HookEvent::LspStart, language_id)
    } else {
        return;
    };
    for script in scripts {
        let _ = state.script.eval(&script);
        let cmds = state.script.drain_commands();
        for cmd in cmds {
            if let crate::scripting::ScriptCommand::LspEnv { pattern, key, value } = cmd {
                if glob_match(&pattern, language_id) {
                    state.lsp.set_env(language_id, key, value);
                }
            }
        }
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    pattern == "*" || pattern == text
}
