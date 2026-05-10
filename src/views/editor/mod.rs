//! EditorView — View wrapper around the Editor core.

mod draw;
mod handle;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::editor::keymap::Keymap;
use crate::editor::Editor;
use crate::highlight::{self, Highlighter};
use crate::settings::EditorSettings;

pub struct EditorView {
    state: ViewState,
    pub editor: Editor,
    path: PathBuf,
    root_dir: PathBuf,
    highlighter: Highlighter,
    file_ext: String,
    pub settings: EditorSettings,
}

impl EditorView {
    pub fn open(path: &Path, settings: &EditorSettings) -> anyhow::Result<Self> {
        let editor = Editor::open(path).map_err(|e| anyhow::anyhow!("{}", e))?;
        let file_ext = highlight::extension_from_path(path).to_string();
        let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let mut view = Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            root_dir,
            highlighter: Highlighter::new(),
            file_ext,
            settings: settings.clone(),
        };
        view.apply_settings();
        Ok(view)
    }

    pub fn new_file(path: &Path, settings: &EditorSettings) -> Self {
        let editor = Editor::from_text("");
        let file_ext = highlight::extension_from_path(path).to_string();
        let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let mut view = Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            root_dir,
            highlighter: Highlighter::new(),
            file_ext,
            settings: settings.clone(),
        };
        view.apply_settings();
        view
    }

    pub fn from_text(content: &str) -> Self {
        let editor = Editor::from_text(content);
        Self {
            state: ViewState::default(),
            editor,
            path: PathBuf::from("[cmd output]"),
            root_dir: PathBuf::from("."),
            highlighter: Highlighter::new(),
            file_ext: String::new(),
            settings: EditorSettings::default(),
        }
    }

    pub fn set_root_dir(&mut self, root: PathBuf) {
        self.root_dir = root;
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn apply_settings(&mut self) {
        self.editor.options.wrap = self.settings.wrap;
        self.editor.options.list = self.settings.list;
        self.editor.options.tab_width = self.settings.tabstop as usize;
        self.editor.options.number = self.settings.number;
    }

    fn gutter_width(&self) -> u16 {
        if !self.editor.options.number {
            return 0;
        }
        let lines = self.editor.buffer.line_count();
        let digits = if lines == 0 {
            1
        } else {
            (lines as f64).log10() as u16 + 1
        };
        digits + 1
    }
}

impl View for EditorView {
    delegate_view_state!(state, override { title, needs_redraw });

    fn title(&self) -> &str {
        self.path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled")
    }

    fn needs_redraw(&self) -> bool {
        true
    }

    fn draw(&self, surface: &mut Surface) {
        self.draw_editor(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };

        let old_mode = self.editor.mode;
        let old_line = self.editor.cursor_line;
        let old_col = self.editor.cursor_col;

        if old_mode == crate::editor::keymap::EditorMode::Command
            || old_mode == crate::editor::keymap::EditorMode::Search
        {
            let result = self.handle_command_input(key, queue);
            self.emit_status_changes(old_mode, old_line, old_col, queue);
            return result;
        }

        let cmd = self.editor.keymap.handle_key(key, self.editor.mode);
        if cmd == crate::editor::command::Command::Noop {
            return HandleResult::Consumed;
        }

        let action = self.editor.execute(cmd);
        self.handle_action(action, queue);
        self.ensure_cursor_visible();
        self.state.dirty = true;
        self.emit_status_changes(old_mode, old_line, old_col, queue);
        HandleResult::Consumed
    }
}
