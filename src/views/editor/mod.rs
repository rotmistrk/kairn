//! EditorView — View wrapper around the Editor core.

mod draw;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::{CM_SAVE, CM_TAB_CLOSE};
use crate::editor::keymap::Keymap;
use crate::editor::{Editor, EditorAction};
use crate::highlight::{self, Highlighter};

pub struct EditorView {
    state: ViewState,
    pub editor: Editor,
    path: PathBuf,
    highlighter: Highlighter,
    file_ext: String,
}

impl EditorView {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let editor = Editor::open(path).map_err(|e| anyhow::anyhow!("{}", e))?;
        let file_ext = highlight::extension_from_path(path).to_string();
        Ok(Self { state: ViewState::default(), editor, path: path.to_path_buf(), highlighter: Highlighter::new(), file_ext })
    }

    pub fn new_file(path: &Path) -> Self {
        let editor = Editor::from_text("");
        let file_ext = highlight::extension_from_path(path).to_string();
        Self { state: ViewState::default(), editor, path: path.to_path_buf(), highlighter: Highlighter::new(), file_ext }
    }

    pub fn from_text(content: &str) -> Self {
        let editor = Editor::from_text(content);
        Self {
            state: ViewState::default(),
            editor,
            path: PathBuf::from("[cmd output]"),
            highlighter: Highlighter::new(),
            file_ext: String::new(),
        }
    }

    pub fn path(&self) -> &Path { &self.path }

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

        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            return self.handle_command_input(key, queue);
        }

        let cmd = self.editor.keymap.handle_key(key, self.editor.mode);
        if cmd == crate::editor::command::Command::Noop { return HandleResult::Consumed; }

        let action = self.editor.execute(cmd);
        self.handle_action(action, queue);
        self.ensure_cursor_visible();
        self.state.dirty = true;
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

    fn complete_command_buf(&mut self) {
        let buf = &self.editor.command_buf;
        let partial = buf.strip_prefix("e ").or_else(|| buf.strip_prefix("edit "));
        let Some(partial) = partial else { return; };
        let root = self.path.parent().unwrap_or(std::path::Path::new("."));
        let search_dir = root;
        let Ok(entries) = std::fs::read_dir(search_dir) else { return; };
        let mut matches: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if name_str.starts_with(partial) {
                matches.push(name_str);
            }
        }
        if matches.len() == 1 {
            let prefix = if buf.starts_with("edit ") { "edit " } else { "e " };
            self.editor.command_buf = format!("{prefix}{}", matches[0]);
        }
    }
}
