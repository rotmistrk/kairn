//! EditorView — View wrapper around the Editor core.

mod build;
mod diff;
pub mod diff_model;
mod draw;
mod draw_blame;
mod draw_diagnostics;
mod draw_diff;
mod draw_style;
mod handle;
mod handle_action;
mod handle_completion;
mod handle_diff;

use std::path::PathBuf;

use txv_core::prelude::*;

use crate::commands::CM_CLIPBOARD_PASTE;
use crate::editor::keymap::Keymap;
use crate::editor::Editor;
use crate::highlight::Highlighter;
use crate::lsp::completion::CompletionPopup;
use crate::settings::EditorSettings;

/// Per-line diff tag for inline diff rendering.
pub struct EditorView {
    state: ViewState,
    pub editor: Editor,
    path: PathBuf,
    root_dir: PathBuf,
    highlighter: Highlighter,
    hl_cache: std::cell::RefCell<crate::highlight_cache::HighlightCache>,
    file_ext: String,
    pub settings: EditorSettings,
    last_edit_tick: u64,
    tick_counter: u64,
    eviction_close: bool,
    display_title: String,
    pub(crate) diagnostics: Option<Vec<crate::lsp::diagnostics::Diagnostic>>,
    /// Blame mode state. None = blame off.
    pub(crate) blame_state: Option<crate::blame::SharedBlame>,
    /// Diff mode state. None = normal mode.
    pub(super) diff_state: Option<diff_model::DiffState>,
    /// Completion popup overlay.
    pub(super) completion_popup: CompletionPopup,
    /// Buffer identity in the shared registry (assigned on open).
    pub buffer_id: Option<crate::buffer_registry::BufferId>,
}

impl View for EditorView {
    delegate_view_state!(state, override { title, needs_redraw, draw });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn needs_redraw(&self) -> bool {
        self.state.is_dirty()
    }

    fn draw(&mut self) {
        self.draw_editor();
        self.draw_blame_gutter();
        self.draw_diagnostics();
        self.completion_popup.draw(&mut self.state.buf);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Tick: autosave check + completion trigger
        if let Event::Tick = event {
            self.handle_tick();
            return HandleResult::Ignored;
        }

        let Event::Key(key) = event else {
            // Handle paste (bracketed paste from terminal)
            if let Event::Paste(text) = event {
                let offset = self
                    .editor
                    .buf()
                    .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                    .unwrap_or(0);
                self.editor.buf().insert(offset, text);
                self.last_edit_tick = self.tick_counter;
                self.state.mark_dirty();
                return HandleResult::Consumed;
            }
            // Handle clipboard paste command
            if let Event::Command { id, data } = event {
                if *id == crate::commands::CM_DIFF {
                    let args = data
                        .as_ref()
                        .and_then(|b| b.downcast_ref::<String>())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    // Side-by-side: create a split with base content on the left
                    if let Some((base_content, base_ref)) = self.try_diff_side_by_side(args) {
                        let payload = crate::commands::DiffSplitRequest { base_content, base_ref };
                        self.state
                            .put_command(crate::commands::CM_DIFF_SPLIT, Some(Box::new(payload)));
                        return HandleResult::Consumed;
                    }
                    self.toggle_diff(args);
                    if !self.editor.status.is_empty() {
                        let msg = txv_core::message::Message::info("editor", self.editor.status.clone());
                        self.state
                            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                    }
                    let mode = if self.in_diff_mode() {
                        "DIFF"
                    } else {
                        "NOR"
                    };
                    self.state
                        .put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new(mode.to_string())));
                    return HandleResult::Consumed;
                }
                if *id == crate::commands::CM_BLAME {
                    self.toggle_blame();
                    return HandleResult::Consumed;
                }
                if *id == crate::commands::CM_DIFF_REVERT {
                    let msg = match self.revert_hunk() {
                        Ok(m) => txv_core::message::Message::info("editor", m),
                        Err(e) => txv_core::message::Message::error("editor", e),
                    };
                    self.state
                        .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                    return HandleResult::Consumed;
                }
                if *id == crate::commands::CM_GOTO_LINE {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() {
                            self.goto(line, col);
                            return HandleResult::Consumed;
                        }
                    }
                }
                if *id == CM_CLIPBOARD_PASTE {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(text) = boxed.downcast_ref::<String>() {
                            let offset = self
                                .editor
                                .buf()
                                .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                                .unwrap_or(0);
                            self.editor.buf().insert(offset, text);
                            self.last_edit_tick = self.tick_counter;
                            self.state.mark_dirty();
                            return HandleResult::Consumed;
                        }
                    }
                }
                if *id == crate::commands::CM_LSP_COMPLETION {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(items) = boxed.downcast_ref::<Vec<crate::lsp::requests::CompletionItem>>() {
                            self.show_completion_items(items);
                            return HandleResult::Consumed;
                        }
                    }
                }
            }
            return HandleResult::Ignored;
        };

        // Completion popup key handling
        if self.completion_popup.visible {
            use txv_core::event::KeyCode;
            match (&key.code, key.modifiers.ctrl) {
                (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
                    self.completion_popup.next();
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
                (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
                    self.completion_popup.prev();
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
                (KeyCode::Enter | KeyCode::Tab, _) => {
                    self.accept_completion();
                    return HandleResult::Consumed;
                }
                (KeyCode::Esc, _) => {
                    self.completion_popup.hide();
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
                _ => {
                    self.completion_popup.hide();
                    // fall through to normal handling
                }
            }
        } else if key.modifiers.ctrl && key.code == txv_core::event::KeyCode::Char('n') {
            // Ctrl+N triggers completion request when popup not visible
            let pos = (
                self.path.clone(),
                self.editor.cursor_line as u32,
                self.editor.cursor_col as u32,
            );
            self.state
                .put_command(crate::commands::CM_LSP_COMPLETION, Some(Box::new(pos)));
            return HandleResult::Consumed;
        }

        // Diff mode: intercept keys for navigation
        if self.in_diff_mode() {
            return self.handle_diff_key(key);
        }

        let old_mode = self.editor.mode;
        let old_line = self.editor.cursor_line;
        let old_col = self.editor.cursor_col;

        if old_mode == crate::editor::keymap::EditorMode::Command
            || old_mode == crate::editor::keymap::EditorMode::Search
        {
            let result = self.handle_command_input(key);
            self.emit_status_changes(old_mode, old_line, old_col);
            return result;
        }

        let cmd = self.editor.keymap.handle_key(key, self.editor.mode);
        if cmd == crate::editor::command::Command::Noop {
            return HandleResult::Consumed;
        }

        let is_search_nav = handle::is_search_navigation(&cmd);
        let action = self.editor.execute(cmd.clone());
        // Track edits for autosave
        if matches!(action, crate::editor::EditorAction::ContentChanged) {
            self.last_edit_tick = self.tick_counter;
            self.hl_cache.borrow_mut().invalidate_from(self.editor.cursor_line);
            // Emit hook triggers for char-inserted / word-completed
            self.emit_hook_triggers(&cmd);
        }
        // Clear highlights on cursor move or content change, except search navigation
        if !is_search_nav
            && matches!(
                action,
                crate::editor::EditorAction::CursorMoved | crate::editor::EditorAction::ContentChanged
            )
        {
            self.editor.highlight = None;
        }
        self.handle_action(action);
        self.ensure_cursor_visible();
        self.state.mark_dirty();
        self.emit_status_changes(old_mode, old_line, old_col);
        self.sync_title();
        HandleResult::Consumed
    }

    fn can_close(&self) -> CloseResult {
        if !self.editor.buf().is_dirty() {
            return CloseResult::Ok;
        }
        if self.settings.autosave {
            return CloseResult::Ok; // will be saved on close
        }
        CloseResult::Denied("unsaved changes".to_string())
    }
}
