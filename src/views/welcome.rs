//! WelcomeView — shown when center slot has no tabs.

use std::path::PathBuf;

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::commands::CM_COMMAND_MODE;
use crate::glyphs::glyphs;
use crate::tool_check::{self, ToolStatus};

pub struct WelcomeView {
    state: ViewState,
    root_dir: PathBuf,
    tools: Option<Vec<ToolStatus>>,
}

impl WelcomeView {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            state: ViewState::default(),
            root_dir,
            tools: None,
        }
    }

    fn ensure_tools(&mut self) {
        if self.tools.is_none() {
            self.tools = Some(tool_check::check_all_tools(&self.root_dir));
        }
    }

    fn build_lines(&self, dim: Style, bright: Style) -> Vec<(String, Style)> {
        let mut lines: Vec<(String, Style)> = vec![
            ("╦╔═╔═╗╦╦═╗╔╗╔".into(), bright),
            ("╠╩╗╠═╣║╠╦╝║║║".into(), bright),
            ("╩ ╩╩ ╩╩╩╚═╝╚╝".into(), bright),
            (String::new(), dim),
            ("Navigate files in tree (F2), Enter to open".into(), dim),
            (":e <file>  to open by name".into(), dim),
            (":q  close tab   :w  save".into(), dim),
            (String::new(), dim),
            ("F1:Help  F5:Zoom  M-x:Command  Ctrl-Q:Quit".into(), dim),
            (":help tutorial  for quick start guide".into(), dim),
        ];

        if let Some(tools) = &self.tools {
            lines.push((String::new(), dim));
            let pal = palette();
            let g = glyphs();
            let green = pal.style(StyleId::StateSuccess);
            let gray = pal.style(StyleId::Dim);
            for tool in tools {
                if tool.found {
                    let ver = tool.version.as_deref().unwrap_or("");
                    lines.push((format!("  {} {} {}", g.check, tool.name, ver), green));
                } else {
                    lines.push((format!("  {} {} — {}", g.cross, tool.name, tool.install_hint), gray));
                }
            }
        }
        lines
    }
}

impl View for WelcomeView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        "Welcome"
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = palette();
        let dim = pal.style(StyleId::Dim);
        let bright = pal.style(StyleId::Bright);

        let lines = self.build_lines(dim, bright);

        let start_y = h.saturating_sub(lines.len() as u16) / 2;
        for row in 0..h {
            let line_i = (row as i32) - (start_y as i32);
            if line_i >= 0 && (line_i as usize) < lines.len() {
                let (ref text, style) = lines[line_i as usize];
                let pad_left = w.saturating_sub(display_width(text, 1)) / 2;
                let left_spaces: String = " ".repeat(pad_left as usize);
                let centered = format!("{}{}", left_spaces, text);
                self.state.buffer_mut().print_line(0, row, &centered, w, style);
            } else {
                self.state.buffer_mut().print_line(0, row, "", w, Style::default());
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Lazy detection on first event (draw is &self, so we do it here)
        self.ensure_tools();

        if let Event::Key(key) = event {
            if key.code() == KeyCode::Char(':') {
                self.state.put_command(CM_COMMAND_MODE, None);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
