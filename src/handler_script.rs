//! Dispatch ScriptCommand variants to the command queue.

use txv_core::message::Message;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::commands::{
    CM_TW_FOCUS_PANEL, CM_TW_LAYOUT_CYCLE, CM_TW_TOGGLE_TOOLS, CM_TW_TOGGLE_TREE, CM_TW_ZOOM,
};
use txv_widgets::{CursorPos, CM_STATUS_MESSAGE};

use crate::app_state::AppState;
use crate::commands::*;
use crate::desktop::SlotId;
use crate::handler::downcast_desktop;
use crate::handler_open::handle_edit_file;
use crate::handler_script_dispatch::dispatch_extended;
use crate::handler_script_edit::{
    handle_clear_highlight, handle_delete_line, handle_replace_selection, handle_replace_word, handle_search,
};
use crate::handler_script_util::fire_hooks_for_event;
use crate::scripting::hooks::HookEvent;
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
        CM_EDITOR_REPLACE_SELECTION => handle_replace_selection(ctx, state),
        CM_EDITOR_DELETE_LINE => handle_delete_line(ctx, state),
        CM_EDITOR_REPLACE_WORD => handle_replace_word(ctx, state),
        CM_CHAR_INSERTED => {
            if let Some(ch) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<char>()) {
                let ch_str = ch.to_string();
                fire_hooks_for_event(state, &HookEvent::CharInserted, &ch_str, ctx);
            }
        }
        CM_WORD_COMPLETED => {
            if let Some(word) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned() {
                fire_hooks_for_event(state, &HookEvent::WordCompleted, &word, ctx);
            }
        }
        CM_EDITOR_SEARCH => {
            if let Some(pattern) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned() {
                handle_search(ctx, state, &pattern);
            }
        }
        CM_EDITOR_CLEAR_HIGHLIGHT => {
            handle_clear_highlight(ctx, state);
        }
        _ => {}
    }
}

fn dispatch_one(cmd: ScriptCommand, ctx: &mut CommandContext, state: &mut AppState) {
    match cmd {
        ScriptCommand::OpenFile { path, line, col } => dispatch_open_file(ctx, state, &path, line, col),
        ScriptCommand::Save => dispatch_save(ctx, state),
        ScriptCommand::SaveAll => ctx.sink.push_command(CM_SAVE_ALL, None),
        ScriptCommand::Close => ctx.sink.push_command(CM_TAB_CLOSE, None),
        ScriptCommand::Goto { line, col } => dispatch_goto(ctx, line, col),
        ScriptCommand::Insert { text } => ctx.sink.push_command(CM_CLIPBOARD_PASTE, Some(Box::new(text))),
        ScriptCommand::Undo | ScriptCommand::Redo => {}
        ScriptCommand::ShowMessage { level, origin, text } => dispatch_show_message(ctx, &level, &origin, &text),
        ScriptCommand::StatusFlash { text } => dispatch_status_flash(ctx, text),
        ScriptCommand::FocusSlot { slot } => dispatch_focus_slot(ctx, &slot),
        ScriptCommand::ViewTheme { mode } => ctx.sink.push_command(CM_TOGGLE_THEME, Some(Box::new(mode))),
        ScriptCommand::ViewZoom => ctx.sink.push_command(CM_TW_ZOOM, None),
        ScriptCommand::ViewToggleTree => ctx.sink.push_command(CM_TW_TOGGLE_TREE, None),
        ScriptCommand::ViewToggleTools => ctx.sink.push_command(CM_TW_TOGGLE_TOOLS, None),
        ScriptCommand::ViewLayout => ctx.sink.push_command(CM_TW_LAYOUT_CYCLE, None),
        ScriptCommand::RunBuild { command } => ctx.sink.push_command(CM_BUILD, command.map(|c| Box::new(c) as _)),
        ScriptCommand::RunTest { command } => ctx.sink.push_command(CM_TEST, command.map(|c| Box::new(c) as _)),
        ScriptCommand::TestFile => ctx.sink.push_command(CM_TEST_FILE, None),
        ScriptCommand::TestAtCursor => ctx.sink.push_command(CM_TEST_AT_CURSOR, None),
        ScriptCommand::NextError => ctx.sink.push_command(CM_NEXT_ERROR, None),
        ScriptCommand::PrevError => ctx.sink.push_command(CM_PREV_ERROR, None),
        ScriptCommand::SetKeyBinding { .. } | ScriptCommand::UnbindKey { .. } => {}
        ScriptCommand::LspHover => ctx.sink.push_command(CM_LSP_HOVER, None),
        ScriptCommand::LspDefinition => ctx.sink.push_command(CM_LSP_GOTO_DEF, None),
        ScriptCommand::LspReferences => ctx.sink.push_command(CM_LSP_FIND_REFS, None),
        ScriptCommand::LspRename { new_name } => ctx.sink.push_command(CM_LSP_RENAME, Some(Box::new(new_name))),
        ScriptCommand::LspFormat => ctx.sink.push_command(CM_LSP_FORMAT, None),
        ScriptCommand::GetSelection | ScriptCommand::GetLine { .. } => {}
        ScriptCommand::ReplaceSelection { .. }
        | ScriptCommand::DeleteLine { .. }
        | ScriptCommand::ReplaceWord { .. }
        | ScriptCommand::Search { .. }
        | ScriptCommand::ClearHighlight => dispatch_editor_edit(cmd, ctx),
        other => {
            dispatch_extended(other, ctx, state);
        }
    }
}

fn dispatch_status_flash(ctx: &mut CommandContext, text: String) {
    ctx.sink
        .push_command(CM_STATUS_MESSAGE, Some(Box::new(Message::info("tcl", text))));
}

fn dispatch_editor_edit(cmd: ScriptCommand, ctx: &mut CommandContext) {
    match cmd {
        ScriptCommand::ReplaceSelection { text } => {
            ctx.sink.push_command(CM_EDITOR_REPLACE_SELECTION, Some(Box::new(text)));
        }
        ScriptCommand::DeleteLine { line } => ctx.sink.push_command(CM_EDITOR_DELETE_LINE, Some(Box::new(line))),
        ScriptCommand::ReplaceWord { text } => ctx.sink.push_command(CM_EDITOR_REPLACE_WORD, Some(Box::new(text))),
        ScriptCommand::Search { pattern } => ctx.sink.push_command(CM_EDITOR_SEARCH, Some(Box::new(pattern))),
        ScriptCommand::ClearHighlight => ctx.sink.push_command(CM_EDITOR_CLEAR_HIGHLIGHT, None),
        _ => {}
    }
}

fn dispatch_open_file(ctx: &mut CommandContext, state: &mut AppState, path: &str, line: Option<u32>, col: Option<u32>) {
    handle_edit_file(ctx.desktop, ctx.sink, state, path);
    let Some(l) = line else {
        return;
    };
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    desktop.focus_panel(SlotId::Center as usize);
    let c = col.unwrap_or(1).saturating_sub(1);
    let editor = desktop
        .panel_mut(SlotId::Center as usize)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<EditorView>());
    if let Some(editor) = editor {
        editor.goto(l.saturating_sub(1), c);
    }
}

fn dispatch_save(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let slot = desktop.focused_panel();
        let editor = desktop
            .panel_mut(slot)
            .and_then(|p| p.active_view_mut())
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_mut::<EditorView>());
        if let Some(editor) = editor {
            if let Err(e) = editor.save() {
                let msg = Message::error("editor", e);
                ctx.sink.push_command(CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
        }
    }
    ctx.sink.push_command(CM_SAVE, None);
    ctx.sink.push_broadcast(CM_FS_CHANGED, None);
    let _ = state;
}

fn dispatch_goto(ctx: &mut CommandContext, line: u32, col: u32) {
    let l = line.saturating_sub(1);
    let c = col.saturating_sub(1);
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let slot = desktop.focused_panel();
    let editor = desktop
        .panel_mut(slot)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<EditorView>());
    if let Some(editor) = editor {
        editor.goto(l, c);
        let pos = CursorPos::new(l + 1, c + 1);
        ctx.sink.push_command(CM_CURSOR_MOVED, Some(Box::new(pos)));
    }
}

fn dispatch_show_message(ctx: &mut CommandContext, level: &str, origin: &str, text: &str) {
    let full_text = format!("[{origin}] {text}");
    let msg = match level {
        "error" => Message::error("tcl", full_text),
        "warn" => Message::warn("tcl", full_text),
        _ => Message::info("tcl", full_text),
    };
    ctx.sink.push_command(CM_STATUS_MESSAGE, Some(Box::new(msg)));
}

fn dispatch_focus_slot(ctx: &mut CommandContext, slot: &str) {
    let panel_id = match slot {
        "left" => 0usize,
        "center" => 1usize,
        "right" => 2usize,
        _ => return,
    };
    ctx.sink.push_command(CM_TW_FOCUS_PANEL, Some(Box::new(panel_id)));
}
