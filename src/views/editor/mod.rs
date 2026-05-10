//! EditorView — View wrapper around the Editor core.

mod draw;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::{CM_SAVE, CM_SET_GLOBAL, CM_TAB_CLOSE};
use crate::editor::keymap::Keymap;
use crate::editor::{Editor, EditorAction};
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
            state: ViewState::default(), editor, path: path.to_path_buf(),
            root_dir, highlighter: Highlighter::new(), file_ext,
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
            state: ViewState::default(), editor, path: path.to_path_buf(),
            root_dir, highlighter: Highlighter::new(), file_ext,
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

    pub fn set_root_dir(&mut self, root: PathBuf) { self.root_dir = root; }

    pub fn path(&self) -> &Path { &self.path }

    fn apply_settings(&mut self) {
        self.editor.options.wrap = self.settings.wrap;
        self.editor.options.list = self.settings.list;
        self.editor.options.tab_width = self.settings.tabstop as usize;
        self.editor.options.number = self.settings.number;
    }

    fn gutter_width(&self) -> u16 {
        if !self.editor.options.number { return 0; }
        let lines = self.editor.buffer.line_count();
        let digits = if lines == 0 { 1 } else { (lines as f64).log10() as u16 + 1 };
        digits + 1
    }
}

impl View for EditorView {
    delegate_view_state!(state, override { title, needs_redraw });

    fn title(&self) -> &str {
        self.path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled")
    }

    fn needs_redraw(&self) -> bool { true }

    fn draw(&self, surface: &mut Surface) { self.draw_editor(surface); }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else { return HandleResult::Ignored; };

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
        if cmd == crate::editor::command::Command::Noop { return HandleResult::Consumed; }

        let action = self.editor.execute(cmd);
        self.handle_action(action, queue);
        self.ensure_cursor_visible();
        self.state.dirty = true;
        self.emit_status_changes(old_mode, old_line, old_col, queue);
        HandleResult::Consumed
    }
}

impl EditorView {
    fn handle_command_input(&mut self, key: &txv_core::event::KeyEvent, queue: &mut EventQueue) -> HandleResult {
        use txv_core::event::KeyCode;
        match &key.code {
            KeyCode::Esc => {
                self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                self.editor.command_buf.clear();
            }
            KeyCode::Enter => {
                let buf = self.editor.command_buf.clone();
                if self.editor.mode == crate::editor::keymap::EditorMode::Search {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                    let action = self.editor.execute(crate::editor::command::Command::SearchForward(buf));
                    self.handle_action(action, queue);
                } else {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                    let action = self.editor.execute(crate::editor::command::Command::ExCommand(buf));
                    self.handle_action(action, queue);
                }
                self.editor.command_buf.clear();
            }
            KeyCode::Backspace => {
                if self.editor.command_buf.is_empty() {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                } else { self.editor.command_buf.pop(); }
            }
            KeyCode::Tab => {
                self.complete_command_buf();
            }
            KeyCode::Char(c) => { self.editor.command_buf.push(*c); }
            _ => {}
        }
        self.ensure_cursor_visible();
        self.state.dirty = true;
        HandleResult::Consumed
    }

    fn handle_action(&mut self, action: EditorAction, queue: &mut EventQueue) {
        match action {
            EditorAction::SaveRequested => {
                let content = self.editor.buffer.content();
                if crate::editor::save::save_file(&self.path, &content).is_ok() {
                    self.editor.buffer.mark_saved();
                }
                queue.put_command(CM_SAVE, None);
            }
            EditorAction::CloseRequested => { queue.put_command(CM_TAB_CLOSE, None); }
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
            _ => {}
        }
    }

    fn ensure_cursor_visible(&mut self) {
        let h = self.state.bounds.h as usize;
        if h == 0 { return; }
        self.editor.viewport_height = h;
        if self.editor.cursor_line < self.editor.viewport_scroll {
            self.editor.viewport_scroll = self.editor.cursor_line;
        } else if self.editor.cursor_line >= self.editor.viewport_scroll + h {
            self.editor.viewport_scroll = self.editor.cursor_line - h + 1;
        }
    }

    fn emit_status_changes(
        &self,
        old_mode: crate::editor::keymap::EditorMode,
        old_line: usize,
        old_col: usize,
        queue: &mut EventQueue,
    ) {
        use crate::commands::{CM_CURSOR_MOVED, CM_MODE_CHANGED};
        use txv_widgets::CursorPos;

        if self.editor.mode != old_mode {
            let name = match self.editor.mode {
                crate::editor::keymap::EditorMode::Normal => "NOR",
                crate::editor::keymap::EditorMode::Insert => "INS",
                crate::editor::keymap::EditorMode::Visual
                | crate::editor::keymap::EditorMode::VisualLine => "VIS",
                crate::editor::keymap::EditorMode::Command => "CMD",
                crate::editor::keymap::EditorMode::Search => "CMD",
            };
            queue.put_command(CM_MODE_CHANGED, Some(Box::new(name.to_string())));
        }
        if self.editor.cursor_line != old_line || self.editor.cursor_col != old_col {
            let pos = CursorPos {
                line: (self.editor.cursor_line + 1) as u32,
                col: (self.editor.cursor_col + 1) as u32,
            };
            queue.put_command(CM_CURSOR_MOVED, Some(Box::new(pos)));
        }
    }

    fn complete_command_buf(&mut self) {
        let buf = &self.editor.command_buf;
        let partial = buf.strip_prefix("e ").or_else(|| buf.strip_prefix("edit "));
        let Some(partial) = partial else { return; };

        let (search_dir, file_prefix, dir_prefix) = if partial.contains('/') {
            let p = Path::new(partial);
            let parent = p.parent().unwrap_or(Path::new(""));
            let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let dp = format!("{}/", parent.display());
            (self.root_dir.join(parent), prefix.to_string(), dp)
        } else {
            (self.root_dir.clone(), partial.to_string(), String::new())
        };

        let Ok(entries) = std::fs::read_dir(&search_dir) else { return; };
        let mut matches: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if name_str.starts_with(&file_prefix) {
                matches.push(format!("{dir_prefix}{name_str}"));
            }
        }
        if matches.len() == 1 {
            let prefix = if buf.starts_with("edit ") { "edit " } else { "e " };
            self.editor.command_buf = format!("{prefix}{}", matches[0]);
        }
    }
}
