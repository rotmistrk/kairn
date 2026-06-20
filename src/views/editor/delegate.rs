//! KairnDelegate — EditorViewDelegate for kairn IDE integration.

use std::mem;
use std::path::PathBuf;

use txv_core::event::{CommandId, KeyEvent};
use txv_core::prelude::*;
use txv_edit::editor::{Editor, EditorAction};
use txv_edit::view::delegate::{EditorViewDelegate, LineDecoration};

use crate::app_palette::app_palette;
use crate::blame::SharedBlame;
use crate::buffer_store::BufferStore;
use crate::completer::AppCompleter;
use crate::gutter_signs::GutterSign;
use crate::lsp::diagnostics::Diagnostic;
use crate::lsp::requests::CompletionItem;
use crate::settings::EditorSettings;

use super::delegate_diff::{diag_marker_style, diag_underline_style};

/// Kairn's delegate: holds IDE state, provides draw info and event hooks.
pub struct KairnDelegate {
    pub(crate) settings: EditorSettings,
    pub(crate) root_dir: PathBuf,
    pub(crate) path: PathBuf,
    pub(crate) file_ext: String,
    pub(crate) display_title: String,
    pub(crate) store: Box<dyn BufferStore>,
    pub(crate) disk_mtime: Option<std::time::SystemTime>,
    pub(crate) last_edit_tick: u64,
    pub(crate) current_tick: u64,
    pub(crate) eviction_close: bool,
    pub(crate) buffer_id: Option<crate::buffer_registry::BufferId>,
    pub(crate) view_id: txv_core::view::ViewId,
    // IDE state
    pub(crate) diagnostics: Option<Vec<Diagnostic>>,
    pub(crate) blame_state: Option<SharedBlame>,
    pub(crate) completion_items: Vec<CompletionItem>,
    pub(crate) completion_visible: bool,
    pub(crate) completion_selected: usize,
    pub(crate) gutter_signs: Vec<(usize, GutterSign)>,
    pub(crate) highlight_word: Option<(usize, usize, usize)>,
    pub(crate) diff_state: Option<super::diff_model::DiffState>,
    pub(crate) command_list: crate::completer::CommandList,
    /// Commands to emit after hook returns (delegate can't reach the view's sink).
    pub(crate) pending_commands: Vec<(u16, Option<Box<dyn std::any::Any + Send>>)>,
    pub(crate) pending_broadcasts: Vec<(u16, Option<Box<dyn std::any::Any + Send>>)>,
    pub(crate) dirty: bool,
    pub(crate) save_requested: bool,
    pub(crate) force_close: bool,
    pub(crate) pending_diff: Option<String>,
    pub(crate) pending_revert: bool,
    pub(crate) pending_nodiff: bool,
    pub(crate) search_hist: Option<txv_core::shared_history::SharedHistory>,
    pub(crate) cmd_hist: Option<txv_core::shared_history::SharedHistory>,
}

impl EditorViewDelegate for KairnDelegate {
    fn extra_gutter_width(&self) -> u16 {
        if !self.settings.number || !self.settings.gutter_signs {
            return self.blame_width();
        }
        2 + self.blame_width()
    }

    fn gutter_sign(&self, line: usize) -> Option<(char, Style)> {
        if !self.show_gutter_signs() {
            return None;
        }
        let app = app_palette();
        self.gutter_signs
            .iter()
            .find(|(l, _)| *l == line)
            .map(|(_, s)| match s {
                GutterSign::Added => ('▎', app.diff().added()),
                GutterSign::Modified => ('▎', app.git().modified()),
                GutterSign::Deleted => ('▸', app.diff().deleted()),
            })
    }

    fn gutter_sign_right(&self, line: usize) -> Option<(char, Style)> {
        if !self.show_gutter_signs() {
            return None;
        }
        let sev = self.diagnostic_severity_at(line)?;
        Some(('●', diag_marker_style(sev)))
    }

    fn extra_style(&self, line: usize, col: usize) -> Option<Style> {
        let diags = self.diagnostics.as_ref()?;
        for d in diags {
            if d.line == line && col >= d.col_start && col < d.col_end {
                return Some(diag_underline_style(d.severity));
            }
        }
        None
    }

    fn line_decorations(&self, _line: usize) -> &[LineDecoration] {
        &[]
    }

    fn highlight_match_style(&self) -> Style {
        app_palette().editor().highlight_match()
    }

    fn highlight_other_bg(&self) -> Color {
        app_palette().editor().highlight_other().bg()
    }

    fn matchparen_style(&self) -> Style {
        app_palette().editor().matchparen()
    }

    fn cursor_render(&self, mode: txv_edit::editor::keymap::EditorMode) -> txv_edit::view::delegate::CursorRender {
        use crate::settings::CursorStyle;
        use txv_edit::editor::keymap::EditorMode;
        use txv_edit::view::delegate::CursorRender;
        let style = match mode {
            EditorMode::Insert => self.settings.cursor_insert,
            _ => self.settings.cursor_normal,
        };
        match style {
            CursorStyle::Software => CursorRender::Software(app_palette().editor().cursor()),
            _ => CursorRender::Hardware,
        }
    }

    fn on_tick(&mut self, editor: &mut Editor, tick: u64) -> HandleResult {
        self.handle_tick(editor, tick)
    }

    fn on_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
        editor: &mut Editor,
    ) -> HandleResult {
        self.handle_command_event(id, data, editor)
    }

    fn on_paste(&mut self, text: &str, editor: &mut Editor) -> HandleResult {
        let offset = editor
            .buf()
            .line_col_to_offset(editor.cursor_line(), editor.cursor_col())
            .unwrap_or(0);
        editor.buf().insert(offset, text);
        self.last_edit_tick = u64::MAX;
        self.clear_diagnostics();
        self.dirty = true;
        HandleResult::Consumed
    }

    fn on_key_pre(&mut self, key: &KeyEvent, editor: &mut Editor) -> Option<HandleResult> {
        if self.highlight_word.is_some() {
            self.highlight_word = None;
            self.dirty = true;
        }
        if let Some(r) = self.handle_diff_key(key) {
            return Some(r);
        }
        self.handle_completion_key(key, editor)
    }

    fn on_action(&mut self, _action: &EditorAction) -> bool {
        false
    }

    fn on_action_post(&mut self, action: &EditorAction, editor: &Editor) {
        self.handle_action_post(action, editor);
    }

    fn on_cursor_moved(&mut self, editor: &Editor) {
        use crate::commands::CM_CURSOR_MOVED;
        use txv_widgets::CursorPos;
        let pos = CursorPos::new((editor.cursor_line() + 1) as u32, (editor.cursor_col() + 1) as u32);
        self.emit(CM_CURSOR_MOVED, Some(Box::new(pos)));
    }

    fn on_mode_changed(
        &mut self,
        _old: txv_edit::editor::keymap::EditorMode,
        new: txv_edit::editor::keymap::EditorMode,
        _editor: &Editor,
    ) {
        use crate::commands::CM_MODE_CHANGED;
        use txv_edit::editor::keymap::EditorMode;
        let name = match new {
            EditorMode::Normal => "NOR",
            EditorMode::Insert => "INS",
            EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock => "VIS",
            EditorMode::Command | EditorMode::Search => "CMD",
            _ => "REP",
        };
        self.emit(CM_MODE_CHANGED, Some(Box::new(name.to_string())));
    }

    fn title(&self, _editor: &Editor) -> Option<&str> {
        Some(&self.display_title)
    }

    fn can_close(&self, editor: &Editor) -> Option<CloseResult> {
        if !editor.buf().is_modified() {
            return Some(CloseResult::Ok);
        }
        if self.settings.autosave {
            return Some(CloseResult::Ok);
        }
        Some(CloseResult::Denied("unsaved changes".to_string()))
    }

    fn needs_redraw(&self, _editor: &Editor) -> bool {
        self.dirty
    }

    fn supports_downcast() -> bool {
        true
    }

    fn cmdline_completer(&self) -> Option<Box<dyn txv_core::complete::Completer>> {
        Some(Box::new(AppCompleter::new(
            self.root_dir.clone(),
            self.command_list.clone(),
        )))
    }

    fn search_history(&self) -> Option<txv_core::shared_history::SharedHistory> {
        self.search_hist.clone()
    }

    fn command_history(&self) -> Option<txv_core::shared_history::SharedHistory> {
        self.cmd_hist.clone()
    }

    fn drain_commands(&mut self) -> Vec<(u16, Option<Box<dyn std::any::Any + Send>>)> {
        mem::take(&mut self.pending_commands)
    }

    fn drain_broadcasts(&mut self) -> Vec<(u16, Option<Box<dyn std::any::Any + Send>>)> {
        mem::take(&mut self.pending_broadcasts)
    }
}
