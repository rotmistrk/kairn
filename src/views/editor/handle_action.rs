//! EditorView action dispatch — maps EditorAction to command queue events.

use txv_core::prelude::*;

use super::EditorView;
use crate::commands::*;
use crate::editor::EditorAction;

impl EditorView {
    pub(super) fn handle_action(&mut self, action: EditorAction, queue: &mut EventQueue) {
        match action {
            EditorAction::SaveRequested => {
                let name = self.path.file_name().unwrap_or(self.path.as_os_str()).to_os_string();
                if self.save_buffer() {
                    queue.put_command(CM_SAVE, None);
                    let msg = txv_core::message::Message::info("editor", format!("Saved: {}", name.to_string_lossy()));
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                } else {
                    let msg = txv_core::message::Message::error(
                        "editor",
                        format!("Failed to save: {}", name.to_string_lossy()),
                    );
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
            }
            EditorAction::CloseRequested => {
                if self.editor.buf().is_dirty() && !self.settings.autosave {
                    self.eviction_close = false;
                    let path = self.path.to_string_lossy().to_string();
                    let ctx = crate::commands::ConfirmContext::EditorClose(path);
                    queue.put_command(crate::commands::CM_SET_CONFIRM_CONTEXT, Some(Box::new(ctx)));
                    queue.put_command(
                        crate::commands::CM_CONFIRM,
                        Some(Box::new("Save changes? [y]es [n]o [Esc]cancel".to_string())),
                    );
                    self.state.mark_dirty();
                } else {
                    if self.settings.autosave && self.editor.buf().is_dirty() {
                        self.save_buffer();
                    }
                    let p = self.path.to_string_lossy().to_string();
                    queue.put_command(crate::commands::CM_FILE_CLOSED, Some(Box::new(p)));
                    queue.put_command(CM_TAB_CLOSE, None);
                }
            }
            EditorAction::ForceCloseRequested => {
                self.editor.buf().mark_saved();
                let p = self.path.to_string_lossy().to_string();
                queue.put_command(crate::commands::CM_FILE_CLOSED, Some(Box::new(p)));
                queue.put_command(CM_TAB_CLOSE, None);
            }
            EditorAction::ShellOutput(output) => {
                queue.put_command(crate::commands::CM_SHELL_OUTPUT, Some(Box::new(output)));
            }
            EditorAction::OpenFile(filename) => {
                let cmd = format!("e {filename}");
                queue.put_command(crate::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            EditorAction::SetGlobal(opt) => {
                queue.put_command(CM_SET_GLOBAL, Some(Box::new(opt)));
            }
            EditorAction::Diff(args) => {
                self.toggle_diff(&args);
                if !self.editor.status.is_empty() {
                    let msg = txv_core::message::Message::info("editor", self.editor.status.clone());
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
                let mode = if self.in_diff_mode() {
                    "DIFF"
                } else {
                    "NOR"
                };
                queue.put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new(mode.to_string())));
            }
            EditorAction::NoDiff => {
                self.exit_diff();
                queue.put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
            }
            EditorAction::LspGotoDefinition => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                queue.put_command(crate::commands::CM_LSP_GOTO_DEF, Some(Box::new(data)));
            }
            EditorAction::LspGotoShow => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                queue.put_command(crate::commands::CM_LSP_GOTO_SHOW, Some(Box::new(data)));
            }
            EditorAction::LspFindReferences => {
                let word = self.editor.word_under_cursor().unwrap_or_default();
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                    word,
                );
                queue.put_command(crate::commands::CM_LSP_FIND_REFS, Some(Box::new(data)));
            }
            EditorAction::LspHover => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                queue.put_command(crate::commands::CM_LSP_HOVER, Some(Box::new(data)));
            }
            EditorAction::AppCommand(cmd) => {
                queue.put_command(crate::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            EditorAction::Split(arg) => {
                let req = crate::commands::SplitRequest {
                    vertical: false,
                    file: if arg.is_empty() {
                        None
                    } else {
                        Some(arg)
                    },
                };
                queue.put_command(crate::commands::CM_SPLIT, Some(Box::new(req)));
            }
            EditorAction::Vsplit(arg) => {
                let req = crate::commands::SplitRequest {
                    vertical: true,
                    file: if arg.is_empty() {
                        None
                    } else {
                        Some(arg)
                    },
                };
                queue.put_command(crate::commands::CM_SPLIT, Some(Box::new(req)));
            }
            EditorAction::Only => {
                queue.put_command(crate::commands::CM_SPLIT_CLOSE, None);
            }
            _ => {}
        }
    }
}
