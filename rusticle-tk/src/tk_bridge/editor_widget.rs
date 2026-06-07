//! Editor widget bridge — creates ViEditor instances from rusticle scripts.
//!
//! Usage from script:
//!   editor create ?-file path?    → returns widget id
//!   editor set $id -content text  → set buffer content
//!   editor get $id                → get buffer content
//!   editor cursor $id             → get "line col"
//!   editor modified $id           → get modified flag

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use txv_edit::editor::keymap::Keymap;
use txv_edit::editor::Editor;

use super::{get_widget, opt_val, parse_opts, require_arg, SendShared};

pub fn register_editor(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("editor", move |_interp, args| {
        let sub = require_arg(args, 0, "editor")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => editor_create(&mut st, args),
            "set" => editor_set(&mut st, args),
            "get" => editor_get(&mut st, args),
            "cursor" => editor_cursor(&mut st, args),
            "modified" => editor_modified(&mut st, args),
            _ => Err(TclError::new(format!("editor: unknown subcommand {sub}"))),
        }
    });
}

fn editor_create(st: &mut super::SharedState, args: &[TclValue]) -> Result<TclValue, TclError> {
    let opts = parse_opts(args, 1);
    let id = st.alloc_id();

    let editor = if let Some(path) = opt_val(&opts, "-file") {
        Editor::open(std::path::Path::new(path)).map_err(|e| TclError::new(format!("editor create: {e}")))?
    } else {
        Editor::from_text("")
    };

    st.desktop
        .insert_widget(id.clone(), Box::new(EditorWidget::new(editor)));
    Ok(TclValue::Str(id))
}

fn editor_set(st: &mut super::SharedState, args: &[TclValue]) -> Result<TclValue, TclError> {
    let id = require_arg(args, 1, "editor set")?;
    let opts = parse_opts(args, 2);
    let ew = get_widget::<EditorWidget>(&mut st.desktop, &id, "editor set")?;
    if let Some(content) = opt_val(&opts, "-content") {
        ew.editor.replace_content(content);
    }
    Ok(TclValue::Str(String::new()))
}

fn editor_get(st: &mut super::SharedState, args: &[TclValue]) -> Result<TclValue, TclError> {
    let id = require_arg(args, 1, "editor get")?;
    let ew = get_widget::<EditorWidget>(&mut st.desktop, &id, "editor get")?;
    Ok(TclValue::Str(ew.editor.buf().content()))
}

fn editor_cursor(st: &mut super::SharedState, args: &[TclValue]) -> Result<TclValue, TclError> {
    let id = require_arg(args, 1, "editor cursor")?;
    let ew = get_widget::<EditorWidget>(&mut st.desktop, &id, "editor cursor")?;
    let line = ew.editor.cursor_line();
    let col = ew.editor.cursor_col();
    Ok(TclValue::Str(format!("{line} {col}")))
}

fn editor_modified(st: &mut super::SharedState, args: &[TclValue]) -> Result<TclValue, TclError> {
    let id = require_arg(args, 1, "editor modified")?;
    let ew = get_widget::<EditorWidget>(&mut st.desktop, &id, "editor modified")?;
    Ok(TclValue::Bool(ew.editor.buf().is_modified()))
}

/// Wrapper that makes Editor into a View for the desktop.
use txv_core::prelude::*;

pub struct EditorWidget {
    state: ViewState,
    pub(crate) editor: Editor,
}

impl EditorWidget {
    pub fn new(editor: Editor) -> Self {
        Self {
            state: ViewState::default(),
            editor,
        }
    }
}

impl View for EditorWidget {
    delegate_view_state!(state);

    fn draw(&mut self) {
        // Minimal draw: just display buffer content line by line
        let buf = self.state.buffer_mut();
        let w = buf.width();
        let h = buf.height();
        let style = Style::default();
        for row in 0..h as usize {
            buf.hline(0, row as u16, w, ' ', style);
            if let Some(line) = self.editor.buf().line(row) {
                let display: String = line.chars().take(w as usize).collect();
                buf.print(0, row as u16, &display, style);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(key) = event {
            let mode = self.editor.mode();
            let cmd = self.editor.keymap_mut().handle_key(key, mode);
            if cmd != txv_edit::editor::command::Command::Noop {
                self.editor.execute(cmd);
            }
            self.state.mark_dirty();
            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }
}
