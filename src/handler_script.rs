//! Dispatch ScriptCommand variants to the command queue.

use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::commands::*;
use crate::layout_group::SlotId;
use crate::scripting::ScriptCommand;
use crate::views::editor::EditorView;

/// Dispatch a list of script commands to the event queue.
pub fn dispatch_script_commands(commands: Vec<ScriptCommand>, ctx: &mut CommandContext, state: &mut AppState) {
    for cmd in commands {
        dispatch_one(cmd, ctx, state);
    }
}

/// Handle script-related commands (CM_EDITOR_REPLACE_SELECTION, CM_CHAR_INSERTED, etc.)
pub fn handle_script_command(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command {
        CM_EDITOR_REPLACE_SELECTION => crate::handler_script_edit::handle_replace_selection(ctx, state),
        CM_EDITOR_DELETE_LINE => crate::handler_script_edit::handle_delete_line(ctx, state),
        CM_EDITOR_REPLACE_WORD => crate::handler_script_edit::handle_replace_word(ctx, state),
        CM_CHAR_INSERTED => {
            if let Some(ch) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<char>()) {
                let ch_str = ch.to_string();
                fire_hooks_for_event(state, &crate::scripting::hooks::HookEvent::CharInserted, &ch_str, ctx);
            }
        }
        CM_WORD_COMPLETED => {
            if let Some(word) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned() {
                fire_hooks_for_event(state, &crate::scripting::hooks::HookEvent::WordCompleted, &word, ctx);
            }
        }
        CM_EDITOR_SEARCH => {
            if let Some(pattern) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned() {
                crate::handler_script_edit::handle_search(ctx, state, &pattern);
            }
        }
        CM_EDITOR_CLEAR_HIGHLIGHT => {
            crate::handler_script_edit::handle_clear_highlight(ctx, state);
        }
        _ => {}
    }
}

fn dispatch_one(cmd: ScriptCommand, ctx: &mut CommandContext, state: &mut AppState) {
    match cmd {
        ScriptCommand::OpenFile { path, line, col } => {
            crate::handler_open::handle_edit_file(ctx.desktop, ctx.sink, state, &path);
            // Focus center after opening
            if let Some(desktop) = crate::handler::downcast_desktop(ctx.desktop) {
                desktop.focus_slot(SlotId::Center);
                // If line/col specified, goto directly (convert 1-indexed to 0-indexed)
                if let Some(l) = line {
                    let c = col.unwrap_or(1).saturating_sub(1);
                    if let Some(view) = desktop.active_view_mut(SlotId::Center) {
                        if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                            editor.goto(l.saturating_sub(1), c);
                        }
                    }
                }
            }
        }
        ScriptCommand::Save => {
            if let Some(desktop) = crate::handler::downcast_desktop(ctx.desktop) {
                let slot = desktop.focused_slot();
                if let Some(view) = desktop.active_view_mut(slot) {
                    if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                        if let Err(e) = editor.save() {
                            let msg = txv_core::message::Message::error("editor", e);
                            ctx.sink
                                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                        }
                    }
                }
            }
            ctx.sink.push_command(CM_SAVE, None);
        }
        ScriptCommand::SaveAll => {
            // Save-all: emit save for current (handler saves all open editors)
            ctx.sink.push_command(CM_SAVE, None);
        }
        ScriptCommand::Close => ctx.sink.push_command(CM_TAB_CLOSE, None),
        ScriptCommand::Goto { line, col } => {
            // Tcl uses 1-indexed; goto() uses 0-indexed
            let l = line.saturating_sub(1);
            let c = col.saturating_sub(1);
            if let Some(desktop) = crate::handler::downcast_desktop(ctx.desktop) {
                let slot = desktop.focused_slot();
                if let Some(view) = desktop.active_view_mut(slot) {
                    if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                        editor.goto(l, c);
                        let pos = txv_widgets::CursorPos {
                            line: l + 1,
                            col: c + 1,
                        };
                        ctx.sink.push_command(CM_CURSOR_MOVED, Some(Box::new(pos)));
                    }
                }
            }
        }
        ScriptCommand::Insert { text } => {
            ctx.sink.push_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
        }
        ScriptCommand::Undo => {} // Handled directly in editor
        ScriptCommand::Redo => {} // Handled directly in editor
        ScriptCommand::ShowMessage { level, origin, text } => {
            let full_text = format!("[{origin}] {text}");
            let msg = match level.as_str() {
                "error" => txv_core::message::Message::error("tcl", full_text),
                "warn" => txv_core::message::Message::warn("tcl", full_text),
                _ => txv_core::message::Message::info("tcl", full_text),
            };
            ctx.sink
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        ScriptCommand::StatusFlash { text } => {
            let msg = txv_core::message::Message::info("tcl", text);
            ctx.sink
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        ScriptCommand::FocusSlot { slot } => {
            let cmd_id = match slot.as_str() {
                "left" => CM_FOCUS_LEFT,
                "center" => CM_FOCUS_CENTER,
                "right" => CM_FOCUS_RIGHT,
                _ => return,
            };
            ctx.sink.push_command(cmd_id, None);
        }
        ScriptCommand::RunBuild { command } => {
            ctx.sink.push_command(CM_BUILD, command.map(|c| Box::new(c) as _));
        }
        ScriptCommand::RunTest { command } => {
            ctx.sink.push_command(CM_TEST, command.map(|c| Box::new(c) as _));
        }
        ScriptCommand::SetKeyBinding { .. } | ScriptCommand::UnbindKey { .. } => {
            // Key bindings are applied at config load time, not at runtime dispatch
        }
        ScriptCommand::LspHover => ctx.sink.push_command(CM_LSP_HOVER, None),
        ScriptCommand::LspDefinition => ctx.sink.push_command(CM_LSP_GOTO_DEF, None),
        ScriptCommand::LspReferences => ctx.sink.push_command(CM_LSP_FIND_REFS, None),
        ScriptCommand::LspRename { new_name } => {
            ctx.sink.push_command(CM_LSP_RENAME, Some(Box::new(new_name)));
        }
        ScriptCommand::LspFormat => {} // No CM_LSP_FORMAT yet
        ScriptCommand::GitStage { file } => {
            ctx.sink.push_command(CM_GIT_STAGE, Some(Box::new(file)));
        }
        ScriptCommand::GitUnstage { file } => {
            ctx.sink.push_command(CM_GIT_UNSTAGE, Some(Box::new(file)));
        }
        ScriptCommand::GitCommit { message } => {
            ctx.sink.push_command(CM_GIT_COMMIT, Some(Box::new(message)));
        }
        ScriptCommand::GitBlame => {
            ctx.sink.push_command(crate::commands::CM_BLAME, None);
        }
        ScriptCommand::TodoAdd { .. } | ScriptCommand::TodoRemove { .. } | ScriptCommand::TodoComplete { .. } => {
            // Todo commands handled via direct tree manipulation
        }
        ScriptCommand::GetSelection | ScriptCommand::GetLine { .. } => {
            // Read operations — handled via snapshot, no command needed
        }
        ScriptCommand::ReplaceSelection { text } => {
            ctx.sink.push_command(CM_EDITOR_REPLACE_SELECTION, Some(Box::new(text)));
        }
        ScriptCommand::DeleteLine { line } => {
            ctx.sink.push_command(CM_EDITOR_DELETE_LINE, Some(Box::new(line)));
        }
        ScriptCommand::ReplaceWord { text } => {
            ctx.sink.push_command(CM_EDITOR_REPLACE_WORD, Some(Box::new(text)));
        }
        ScriptCommand::Search { pattern } => {
            ctx.sink.push_command(CM_EDITOR_SEARCH, Some(Box::new(pattern)));
        }
        ScriptCommand::ClearHighlight => {
            ctx.sink.push_command(CM_EDITOR_CLEAR_HIGHLIGHT, None);
        }
        ScriptCommand::SplitVertical { file } => {
            let req = crate::commands::SplitRequest { vertical: true, file };
            ctx.sink.push_command(CM_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitHorizontal { file } => {
            let req = crate::commands::SplitRequest { vertical: false, file };
            ctx.sink.push_command(CM_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitClose => {
            ctx.sink.push_command(CM_SPLIT_CLOSE, None);
        }
        ScriptCommand::SplitFocus => {
            ctx.sink.push_command(CM_SPLIT_FOCUS, None);
        }
        ScriptCommand::SplitOpen { path } => {
            let req = crate::commands::OpenFileRequest {
                path: std::path::PathBuf::from(path),
                line: None,
                col: None,
                diff: false,
            };
            ctx.sink.push_command(CM_OPEN_IN_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::DiffRevert => {
            ctx.sink.push_command(CM_DIFF_REVERT, None);
        }
    }
}

/// Slot name to SlotId conversion.
pub fn slot_from_name(name: &str) -> Option<SlotId> {
    match name {
        "left" => Some(SlotId::Left),
        "center" => Some(SlotId::Center),
        "right" => Some(SlotId::Right),
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
