//! Action post-processing — emits kairn commands from editor actions.

use std::path::PathBuf;

use txv_edit::editor::EditorAction;

use super::delegate::KairnDelegate;
use crate::commands::*;
use crate::editor::Editor;

impl KairnDelegate {
    pub(crate) fn handle_action_post(&mut self, action: &EditorAction, _editor: &Editor) {
        match action {
            EditorAction::ContentChanged => {
                self.last_edit_tick = u64::MAX;
                self.clear_diagnostics();
            }
            EditorAction::SaveRequested => self.save_requested = true,
            EditorAction::CloseRequested => self.emit(CM_TAB_CLOSE, None),
            EditorAction::ForceCloseRequested => self.force_close = true,
            EditorAction::ShellOutput(output) => {
                self.emit(CM_SHELL_OUTPUT, Some(Box::new(output.clone())));
            }
            EditorAction::OpenFile(filename) => {
                self.emit(CM_EXECUTE_COMMAND, Some(Box::new(format!("e {filename}"))));
            }
            EditorAction::SetGlobal(opt) => self.emit(CM_SET_GLOBAL, Some(Box::new(opt.clone()))),
            EditorAction::AppCommand(cmd) => self.emit(CM_EXECUTE_COMMAND, Some(Box::new(cmd.clone()))),
            EditorAction::BuiltinFormat(args) => {
                self.emit(CM_EXECUTE_COMMAND, Some(Box::new(format!("fmt! {args}"))));
            }
            _ => self.handle_action_extended(action),
        }
    }

    fn handle_action_extended(&mut self, action: &EditorAction) {
        match action {
            EditorAction::Diff(args) => self.pending_diff = Some(args.clone()),
            EditorAction::NoDiff => {
                self.diff_state = None;
                self.pending_nodiff = true;
                self.dirty = true;
            }
            EditorAction::NoBlame => {
                self.blame_state = None;
                self.dirty = true;
            }
            EditorAction::Revert => self.pending_revert = true,
            EditorAction::LspGotoDefinition => self.emit_lsp_pos(CM_LSP_GOTO_DEF),
            EditorAction::LspGotoShow => self.emit_lsp_pos(CM_LSP_GOTO_SHOW),
            EditorAction::LspFindReferences => self.emit_lsp_pos(CM_LSP_FIND_REFS),
            EditorAction::LspHover => self.emit_lsp_pos(CM_LSP_HOVER),
            EditorAction::Split(arg) => self.emit_split(arg, false),
            EditorAction::Vsplit(arg) => self.emit_split(arg, true),
            EditorAction::Only => self.emit(CM_SPLIT_CLOSE, None),
            EditorAction::LspFormat => {
                let data: (PathBuf, Option<(u32, u32)>, u32) = (self.path.clone(), None, 4);
                self.emit(CM_LSP_FORMAT, Some(Box::new(data)));
            }
            EditorAction::LspFormatRange(start, end) => {
                let data: (PathBuf, Option<(u32, u32)>, u32) =
                    (self.path.clone(), Some((*start as u32, *end as u32)), 4);
                self.emit(CM_LSP_FORMAT, Some(Box::new(data)));
            }
            _ => {}
        }
    }

    fn emit_split(&mut self, arg: &str, vertical: bool) {
        let file = if arg.is_empty() {
            None
        } else {
            Some(arg.to_string())
        };
        let req = SplitRequest { vertical, file };
        self.emit(CM_SPLIT, Some(Box::new(req)));
    }

    fn emit_lsp_pos(&mut self, cmd_id: u16) {
        let data = (self.path.clone(), 0u32, 0u32);
        self.emit(cmd_id, Some(Box::new(data)));
    }
}
