//! EditorView — View wrapper around the Editor core.

mod draw;
mod handle;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::CM_CLIPBOARD_PASTE;
use crate::commands::CM_TAB_CLOSE;
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
    last_edit_tick: u64,
    tick_counter: u64,
    close_prompt: bool,
    display_title: String,
}

impl EditorView {
    pub fn open(path: &Path, settings: &EditorSettings) -> anyhow::Result<Self> {
        let editor = Editor::open(path).map_err(|e| anyhow::anyhow!("{}", e))?;
        let file_ext = highlight::extension_from_path(path).to_string();
        let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let display_title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        let mut view = Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            root_dir,
            highlighter: Highlighter::new(),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            close_prompt: false,
            display_title,
        };
        view.apply_settings();
        Ok(view)
    }

    pub fn new_file(path: &Path, settings: &EditorSettings) -> Self {
        let editor = Editor::from_text("");
        let file_ext = highlight::extension_from_path(path).to_string();
        let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let display_title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        let mut view = Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            root_dir,
            highlighter: Highlighter::new(),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            close_prompt: false,
            display_title,
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
            last_edit_tick: 0,
            tick_counter: 0,
            close_prompt: false,
            display_title: "[cmd output]".to_string(),
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
        &self.display_title
    }

    fn needs_redraw(&self) -> bool {
        true
    }

    fn draw(&self, surface: &mut Surface) {
        self.draw_editor(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Tick: autosave check
        if let Event::Tick = event {
            self.tick_counter += 1;
            if self.settings.autosave
                && self.last_edit_tick > 0
                && self.tick_counter - self.last_edit_tick >= self.settings.autosave_delay as u64
            {
                self.last_edit_tick = 0;
                if self.editor.buffer.is_dirty() {
                    let content = self.editor.buffer.content();
                    if crate::editor::save::save_file(&self.path, &content).is_ok() {
                        self.editor.buffer.mark_saved();
                        self.sync_title();
                    }
                }
            }
            return HandleResult::Ignored;
        }

        let Event::Key(key) = event else {
            // Handle paste (bracketed paste from terminal)
            if let Event::Paste(text) = event {
                let offset = self
                    .editor
                    .buffer
                    .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                    .unwrap_or(0);
                self.editor.buffer.insert(offset, text);
                self.last_edit_tick = self.tick_counter;
                self.state.dirty = true;
                return HandleResult::Consumed;
            }
            // Handle clipboard paste command
            if let Event::Command { id, data } = event {
                if *id == CM_CLIPBOARD_PASTE {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(text) = boxed.downcast_ref::<String>() {
                            let offset = self
                                .editor
                                .buffer
                                .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                                .unwrap_or(0);
                            self.editor.buffer.insert(offset, text);
                            self.last_edit_tick = self.tick_counter;
                            self.state.dirty = true;
                            return HandleResult::Consumed;
                        }
                    }
                }
            }
            return HandleResult::Ignored;
        };

        // Close prompt: y/n/c
        if self.close_prompt {
            use txv_core::event::KeyCode;
            match &key.code {
                KeyCode::Char('y') => {
                    self.close_prompt = false;
                    let content = self.editor.buffer.content();
                    let _ = crate::editor::save::save_file(&self.path, &content);
                    self.editor.buffer.mark_saved();
                    queue.put_command(CM_TAB_CLOSE, None);
                }
                KeyCode::Char('n') => {
                    self.close_prompt = false;
                    self.editor.buffer.mark_saved(); // discard
                    queue.put_command(CM_TAB_CLOSE, None);
                }
                _ => {
                    self.close_prompt = false;
                    self.editor.status = String::new();
                }
            }
            self.state.dirty = true;
            return HandleResult::Consumed;
        }

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
        // Track edits for autosave
        if matches!(action, crate::editor::EditorAction::ContentChanged) {
            self.last_edit_tick = self.tick_counter;
        }
        self.handle_action(action, queue);
        self.ensure_cursor_visible();
        self.state.dirty = true;
        self.emit_status_changes(old_mode, old_line, old_col, queue);
        self.sync_title();
        HandleResult::Consumed
    }

    fn can_close(&self) -> CloseResult {
        if !self.editor.buffer.is_dirty() {
            return CloseResult::Ok;
        }
        if self.settings.autosave {
            return CloseResult::Ok; // will be saved on close
        }
        CloseResult::Denied("unsaved changes".to_string())
    }
}
