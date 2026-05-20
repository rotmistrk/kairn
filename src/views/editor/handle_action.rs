//! EditorView action dispatch — maps EditorAction to command queue events.

use super::EditorView;
use crate::commands::*;
use crate::editor::EditorAction;

impl EditorView {
    pub(super) fn handle_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::SaveRequested => {
                let name = self.path.file_name().unwrap_or(self.path.as_os_str()).to_os_string();
                if self.save_buffer() {
                    self.state.put_command(CM_SAVE, None);
                    let msg = txv_core::message::Message::info("editor", format!("Saved: {}", name.to_string_lossy()));
                    self.state
                        .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                } else {
                    let msg = txv_core::message::Message::error(
                        "editor",
                        format!("Failed to save: {}", name.to_string_lossy()),
                    );
                    self.state
                        .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
            }
            EditorAction::CloseRequested => {
                if self.editor.buf().is_dirty() && !self.settings.autosave {
                    self.eviction_close = false;
                    let path = self.path.to_string_lossy().to_string();
                    let ctx = crate::commands::ConfirmContext::EditorClose(path);
                    self.state
                        .put_command(crate::commands::CM_SET_CONFIRM_CONTEXT, Some(Box::new(ctx)));
                    self.state.put_command(
                        crate::commands::CM_CONFIRM,
                        Some(Box::new("Save changes? [y]es [n]o [Esc]cancel".to_string())),
                    );
                    self.state.mark_dirty();
                } else {
                    if self.settings.autosave && self.editor.buf().is_dirty() {
                        self.save_buffer();
                    }
                    let p = self.path.to_string_lossy().to_string();
                    self.state
                        .put_command(crate::commands::CM_FILE_CLOSED, Some(Box::new(p)));
                    self.state.put_command(CM_TAB_CLOSE, None);
                }
            }
            EditorAction::ForceCloseRequested => {
                self.editor.buf().mark_saved();
                let p = self.path.to_string_lossy().to_string();
                self.state
                    .put_command(crate::commands::CM_FILE_CLOSED, Some(Box::new(p)));
                self.state.put_command(CM_TAB_CLOSE, None);
            }
            EditorAction::ShellOutput(output) => {
                self.state
                    .put_command(crate::commands::CM_SHELL_OUTPUT, Some(Box::new(output)));
            }
            EditorAction::OpenFile(filename) => {
                let cmd = format!("e {filename}");
                self.state
                    .put_command(crate::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            EditorAction::SetGlobal(opt) => {
                self.state.put_command(CM_SET_GLOBAL, Some(Box::new(opt)));
            }
            EditorAction::Diff(args) => {
                if let Some((base_content, base_ref)) = self.try_diff_side_by_side(&args) {
                    let payload = crate::commands::DiffSplitRequest { base_content, base_ref };
                    self.state
                        .put_command(crate::commands::CM_DIFF_SPLIT, Some(Box::new(payload)));
                    return;
                }
                self.toggle_diff(&args);
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
            }
            EditorAction::NoDiff => {
                self.exit_diff();
                self.state
                    .put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
            }
            EditorAction::NoBlame => {
                self.blame_state = None;
                self.state.mark_dirty();
            }
            EditorAction::Revert => {
                let msg = match self.revert_hunk() {
                    Ok(m) => txv_core::message::Message::info("editor", m),
                    Err(e) => txv_core::message::Message::error("editor", e),
                };
                self.state
                    .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
            EditorAction::LspGotoDefinition => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                self.state
                    .put_command(crate::commands::CM_LSP_GOTO_DEF, Some(Box::new(data)));
            }
            EditorAction::LspGotoShow => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                self.state
                    .put_command(crate::commands::CM_LSP_GOTO_SHOW, Some(Box::new(data)));
            }
            EditorAction::LspFindReferences => {
                let word = self.editor.word_under_cursor().unwrap_or_default();
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                    word,
                );
                self.state
                    .put_command(crate::commands::CM_LSP_FIND_REFS, Some(Box::new(data)));
            }
            EditorAction::LspHover => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                self.state
                    .put_command(crate::commands::CM_LSP_HOVER, Some(Box::new(data)));
            }
            EditorAction::AppCommand(cmd) => {
                self.state
                    .put_command(crate::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
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
                self.state.put_command(crate::commands::CM_SPLIT, Some(Box::new(req)));
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
                self.state.put_command(crate::commands::CM_SPLIT, Some(Box::new(req)));
            }
            EditorAction::Only => {
                self.state.put_command(crate::commands::CM_SPLIT_CLOSE, None);
            }
            _ => {}
        }
    }
}
