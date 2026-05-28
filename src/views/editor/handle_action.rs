//! EditorView action dispatch — maps EditorAction to command queue events.

use txv_core::message::Message;

use super::EditorView;
use crate::commands::*;
use crate::editor::EditorAction;

impl EditorView {
    pub(super) fn handle_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::SaveRequested => self.action_save(),
            EditorAction::CloseRequested => self.action_close(),
            EditorAction::ForceCloseRequested => self.action_force_close(),
            EditorAction::ShellOutput(output) => {
                self.state.put_command(CM_SHELL_OUTPUT, Some(Box::new(output)));
            }
            EditorAction::OpenFile(filename) => {
                let cmd = format!("e {filename}");
                self.state.put_command(CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            EditorAction::SetGlobal(opt) => {
                self.state.put_command(CM_SET_GLOBAL, Some(Box::new(opt)));
            }
            EditorAction::Diff(args) => self.action_diff(&args),
            EditorAction::NoDiff => self.action_no_diff(),
            EditorAction::NoBlame => {
                self.blame_state = None;
                self.state.mark_dirty();
            }
            EditorAction::Revert => self.action_revert(),
            EditorAction::LspGotoDefinition => self.action_lsp_goto(CM_LSP_GOTO_DEF),
            EditorAction::LspGotoShow => self.action_lsp_goto(CM_LSP_GOTO_SHOW),
            EditorAction::LspFindReferences => self.action_lsp_find_refs(),
            EditorAction::LspHover => self.action_lsp_goto(CM_LSP_HOVER),
            EditorAction::AppCommand(cmd) => {
                self.state.put_command(CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            EditorAction::Split(arg) => self.action_split(arg, false),
            EditorAction::Vsplit(arg) => self.action_split(arg, true),
            EditorAction::Only => {
                self.state.put_command(CM_SPLIT_CLOSE, None);
            }
            _ => {}
        }
    }

    fn action_save(&mut self) {
        let name = self.path.file_name().unwrap_or(self.path.as_os_str()).to_os_string();
        if self.save_buffer() {
            self.state.put_command(CM_SAVE, None);
            let msg = Message::info("editor", format!("Saved: {}", name.to_string_lossy()));
            self.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        } else {
            let msg = Message::error("editor", format!("Failed to save: {}", name.to_string_lossy()));
            self.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }

    fn action_close(&mut self) {
        if self.editor.buf().is_dirty() && !self.settings.autosave {
            self.eviction_close = false;
            let path = self.path.to_string_lossy().to_string();
            let ctx = ConfirmContext::EditorClose(path);
            self.state.put_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ctx)));
            self.state.put_command(
                CM_CONFIRM,
                Some(Box::new("Save changes? [y]es [n]o [Esc]cancel".to_string())),
            );
            self.state.mark_dirty();
        } else {
            if self.settings.autosave && self.editor.buf().is_dirty() {
                self.save_buffer();
            }
            let p = self.path.to_string_lossy().to_string();
            self.state.put_command(CM_FILE_CLOSED, Some(Box::new(p)));
            self.state.put_command(CM_TAB_CLOSE, None);
        }
    }

    fn action_force_close(&mut self) {
        self.editor.buf().mark_saved();
        let p = self.path.to_string_lossy().to_string();
        self.state.put_command(CM_FILE_CLOSED, Some(Box::new(p)));
        self.state.put_command(CM_TAB_CLOSE, None);
    }

    fn action_diff(&mut self, args: &str) {
        if let Some((base_content, base_ref)) = self.try_diff_side_by_side(args) {
            let payload = DiffSplitRequest { base_content, base_ref };
            self.state.put_command(CM_DIFF_SPLIT, Some(Box::new(payload)));
            return;
        }
        self.toggle_diff(args);
        if !self.editor.status.is_empty() {
            let msg = Message::info("editor", self.editor.status.clone());
            self.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        let mode = if self.in_diff_mode() {
            "DIFF"
        } else {
            "NOR"
        };
        self.state
            .put_command(CM_MODE_CHANGED, Some(Box::new(mode.to_string())));
    }

    fn action_no_diff(&mut self) {
        self.exit_diff();
        self.state
            .put_command(CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
    }

    fn action_revert(&mut self) {
        let msg = match self.revert_hunk() {
            Ok(m) => Message::info("editor", m),
            Err(e) => Message::error("editor", e),
        };
        self.state
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }

    fn action_lsp_goto(&mut self, cmd_id: u16) {
        let data = (
            self.path.clone(),
            self.editor.cursor_line as u32,
            self.editor.cursor_col as u32,
        );
        self.state.put_command(cmd_id, Some(Box::new(data)));
    }

    fn action_lsp_find_refs(&mut self) {
        let word = self.editor.word_under_cursor().unwrap_or_default();
        let data = (
            self.path.clone(),
            self.editor.cursor_line as u32,
            self.editor.cursor_col as u32,
            word,
        );
        self.state.put_command(CM_LSP_FIND_REFS, Some(Box::new(data)));
    }

    fn action_split(&mut self, arg: String, vertical: bool) {
        let req = SplitRequest {
            vertical,
            file: if arg.is_empty() {
                None
            } else {
                Some(arg)
            },
        };
        self.state.put_command(CM_SPLIT, Some(Box::new(req)));
    }
}
