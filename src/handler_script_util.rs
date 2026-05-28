//! Script utility functions: hook firing, slot lookup, LSP command dispatch.

use txv_core::message::Message;
use txv_core::program::CommandContext;
use txv_widgets::CM_STATUS_MESSAGE;

use crate::app_state::AppState;
use crate::desktop::SlotId;
use crate::handler_lsp_cmd::handle_lsp_command as lsp_handle;
use crate::handler_script::dispatch_script_commands;
use crate::scripting::hooks::HookEvent;
use crate::scripting::ScriptCommand;

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
pub fn fire_hooks_for_event(state: &mut AppState, event: &HookEvent, context: &str, ctx: &mut CommandContext) {
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
    let msg = lsp_handle(arg, state);
    ctx.sink
        .push_command(CM_STATUS_MESSAGE, Some(Box::new(Message::info("lsp", msg))));
}

/// Fire lsp-start hook for a language. Runs synchronously (no CommandContext needed).
pub fn fire_lsp_start_hook(state: &mut AppState, language_id: &str) {
    let scripts = if let Ok(reg) = state.script.hook_registry.lock() {
        reg.fire(&HookEvent::LspStart, language_id)
    } else {
        return;
    };
    for script in scripts {
        let _ = state.script.eval(&script);
        let cmds = state.script.drain_commands();
        apply_lsp_env_commands(cmds, state, language_id);
    }
}

fn apply_lsp_env_commands(cmds: Vec<ScriptCommand>, state: &mut AppState, language_id: &str) {
    for cmd in cmds {
        let ScriptCommand::LspEnv { pattern, key, value } = cmd else {
            continue;
        };
        if glob_match(&pattern, language_id) {
            state.lsp.set_env(language_id, key, value);
        }
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    pattern == "*" || pattern == text
}
