//! Dispatch ScriptCommand variants to the command queue.

use txv_core::program::CommandContext;
use txv_widgets::CM_STATUS_MESSAGE;

use crate::app_state::AppState;
use crate::commands::*;
use crate::desktop::SlotId;
use crate::handler_script_util::fire_hooks_for_event;
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
            if let Some(desktop) = crate::handler::downcast_desktop(ctx.desktop) {
                desktop.focus_panel(SlotId::Center as usize);
                if let Some(l) = line {
                    let c = col.unwrap_or(1).saturating_sub(1);
                    if let Some(panel) = desktop.panel_mut(SlotId::Center as usize) {
                        if let Some(view) = panel.active_view_mut() {
                            if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                                editor.goto(l.saturating_sub(1), c);
                            }
                        }
                    }
                }
            }
        }
        ScriptCommand::Save => {
            if let Some(desktop) = crate::handler::downcast_desktop(ctx.desktop) {
                let slot = desktop.focused_panel();
                if let Some(panel) = desktop.panel_mut(slot) {
                    if let Some(view) = panel.active_view_mut() {
                        if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                            if let Err(e) = editor.save() {
                                let msg = txv_core::message::Message::error("editor", e);
                                ctx.sink.push_command(CM_STATUS_MESSAGE, Some(Box::new(msg)));
                            }
                        }
                    }
                }
            }
            ctx.sink.push_command(CM_SAVE, None);
        }
        ScriptCommand::SaveAll => {
            ctx.sink.push_command(CM_SAVE_ALL, None);
        }
        ScriptCommand::Close => ctx.sink.push_command(CM_TAB_CLOSE, None),
        ScriptCommand::Goto { line, col } => {
            let l = line.saturating_sub(1);
            let c = col.saturating_sub(1);
            if let Some(desktop) = crate::handler::downcast_desktop(ctx.desktop) {
                let slot = desktop.focused_panel();
                if let Some(panel) = desktop.panel_mut(slot) {
                    if let Some(view) = panel.active_view_mut() {
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
        }
        ScriptCommand::Insert { text } => {
            ctx.sink.push_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
        }
        ScriptCommand::Undo | ScriptCommand::Redo => {}
        ScriptCommand::ShowMessage { level, origin, text } => {
            let full_text = format!("[{origin}] {text}");
            let msg = match level.as_str() {
                "error" => txv_core::message::Message::error("tcl", full_text),
                "warn" => txv_core::message::Message::warn("tcl", full_text),
                _ => txv_core::message::Message::info("tcl", full_text),
            };
            ctx.sink.push_command(CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        ScriptCommand::StatusFlash { text } => {
            let msg = txv_core::message::Message::info("tcl", text);
            ctx.sink.push_command(CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        ScriptCommand::FocusSlot { slot } => {
            let panel_id = match slot.as_str() {
                "left" => 0usize,
                "center" => 1usize,
                "right" => 2usize,
                _ => return,
            };
            ctx.sink.push_command(
                txv_widgets::tiled_workspace::commands::CM_TW_FOCUS_PANEL,
                Some(Box::new(panel_id)),
            );
        }
        ScriptCommand::ViewTheme { mode } => {
            ctx.sink.push_command(CM_TOGGLE_THEME, Some(Box::new(mode)));
        }
        ScriptCommand::ViewZoom => {
            ctx.sink
                .push_command(txv_widgets::tiled_workspace::commands::CM_TW_ZOOM, None);
        }
        ScriptCommand::ViewToggleTree => {
            ctx.sink
                .push_command(txv_widgets::tiled_workspace::commands::CM_TW_TOGGLE_TREE, None);
        }
        ScriptCommand::ViewToggleTools => {
            ctx.sink
                .push_command(txv_widgets::tiled_workspace::commands::CM_TW_TOGGLE_TOOLS, None);
        }
        ScriptCommand::ViewLayout => {
            ctx.sink
                .push_command(txv_widgets::tiled_workspace::commands::CM_TW_LAYOUT_CYCLE, None);
        }
        ScriptCommand::RunBuild { command } => {
            ctx.sink.push_command(CM_BUILD, command.map(|c| Box::new(c) as _));
        }
        ScriptCommand::RunTest { command } => {
            ctx.sink.push_command(CM_TEST, command.map(|c| Box::new(c) as _));
        }
        ScriptCommand::TestFile => ctx.sink.push_command(CM_TEST_FILE, None),
        ScriptCommand::TestAtCursor => ctx.sink.push_command(CM_TEST_AT_CURSOR, None),
        ScriptCommand::NextError => ctx.sink.push_command(CM_NEXT_ERROR, None),
        ScriptCommand::PrevError => ctx.sink.push_command(CM_PREV_ERROR, None),
        ScriptCommand::SetKeyBinding { .. } | ScriptCommand::UnbindKey { .. } => {}
        ScriptCommand::LspHover => ctx.sink.push_command(CM_LSP_HOVER, None),
        ScriptCommand::LspDefinition => ctx.sink.push_command(CM_LSP_GOTO_DEF, None),
        ScriptCommand::LspReferences => ctx.sink.push_command(CM_LSP_FIND_REFS, None),
        ScriptCommand::LspRename { new_name } => {
            ctx.sink.push_command(CM_LSP_RENAME, Some(Box::new(new_name)));
        }
        ScriptCommand::LspFormat => {}
        ScriptCommand::GetSelection | ScriptCommand::GetLine { .. } => {}
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
        other => {
            crate::handler_script_dispatch::dispatch_extended(other, ctx, state);
        }
    }
}
