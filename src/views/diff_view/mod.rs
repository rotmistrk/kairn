//! DiffView — standalone diff viewer that replaces editor tab during diff mode.
//!
//! Uses GroupState to host an InputLine for command mode (: prefix),
//! same pattern as txv-edit's EditorView.

mod draw;
mod handle;

use std::path::PathBuf;

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::views::editor::diff_model::DiffState;

/// Standalone diff view replacing the editor tab.
pub struct DiffView {
    group: GroupState,
    ds: DiffState,
    buf_lines: Vec<String>,
    path: PathBuf,
    show_numbers: bool,
    display_title: String,
    cmd_active: bool,
    cmdline_prefix: char,
}

impl DiffView {
    pub fn new(ds: DiffState, buf_content: &str, path: PathBuf, show_numbers: bool) -> Self {
        let buf_lines: Vec<String> = buf_content.lines().map(|l| l.to_string()).collect();
        let name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let display_title = format!("[diff] {name}");
        Self {
            group: GroupState::new(ViewOptions::default()),
            ds,
            buf_lines,
            path,
            show_numbers,
            display_title,
            cmd_active: false,
            cmdline_prefix: ':',
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn cursor_buf_line(&self) -> usize {
        self.ds.cursor_buf_line()
    }

    fn height(&self) -> usize {
        self.group.bounds().h() as usize
    }

    fn width(&self) -> u16 {
        self.group.bounds().w()
    }

    fn content_height(&self) -> usize {
        if self.cmd_active {
            self.height().saturating_sub(1)
        } else {
            self.height()
        }
    }

    fn activate_cmdline(&mut self) {
        use txv_widgets::input_line::InputLine;
        let il = InputLine::new();
        self.group.insert(Box::new(il));
        self.group.set_focused_index(0);
        self.group.select_focused();
        self.cmd_active = true;
        self.cmdline_prefix = ':';
        self.relayout_cmdline();
        self.group.mark_dirty();
    }

    fn deactivate_cmdline(&mut self) {
        if self.cmd_active {
            self.group.remove(0);
            self.cmd_active = false;
            self.group.mark_dirty();
        }
    }

    fn relayout_cmdline(&mut self) {
        if !self.cmd_active {
            return;
        }
        let b = self.group.bounds();
        let y = b.h().saturating_sub(1);
        let prefix_w = 1u16;
        let input_w = b.w().saturating_sub(prefix_w);
        self.group.set_child_bounds(0, Rect::new(prefix_w, y, input_w, 1));
    }

    fn cmdline_text(&mut self) -> String {
        use txv_widgets::input_line::InputLine;
        self.group
            .child_mut(0)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_ref::<InputLine>())
            .map(|il| il.text().to_string())
            .unwrap_or_default()
    }
}

impl View for DiffView {
    delegate_group_state!(group, override { set_bounds, draw, handle, title, needs_redraw, select, unselect });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn needs_redraw(&self) -> bool {
        self.group.any_dirty()
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.group.mark_dirty();
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.group.mark_dirty();
    }

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.relayout_cmdline();
    }

    fn draw(&mut self) {
        self.draw_unified();
        if self.cmd_active {
            let b = self.group.bounds();
            let y = b.h().saturating_sub(1);
            let style = palette().style(StyleId::StatusBar);
            self.group.buffer_mut().put(0, y, self.cmdline_prefix, style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if self.cmd_active {
            return self.handle_cmdline_event(event);
        }
        match event {
            Event::Key(key) => self.handle_key(key),
            _ => HandleResult::Ignored,
        }
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
