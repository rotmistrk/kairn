//! WelcomeView — shown when center slot has no tabs.

use std::path::PathBuf;

use txv_core::cell::{Color, Style};
use txv_core::prelude::*;

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
            self.tools = Some(tool_check::check_tools(&self.root_dir));
        }
    }
}

impl View for WelcomeView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        "Welcome"
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let pal = txv_core::palette::palette();
        let dim = pal.base.dim.to_style();
        let bright = pal.base.bright.to_style();

        // Build all lines to render
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
        ];

        // Add tool checklist
        if let Some(tools) = &self.tools {
            lines.push((String::new(), dim));
            let g = glyphs();
            let green = Style {
                fg: Color::Ansi(2),
                ..Style::default()
            };
            let gray = Style {
                fg: Color::Ansi(8),
                ..Style::default()
            };
            for tool in tools {
                if tool.found {
                    let ver = tool.version.as_deref().unwrap_or("");
                    lines.push((format!("  {} {} {}", g.check, tool.name, ver), green));
                } else {
                    lines.push((format!("  {} {} — {}", g.cross, tool.name, tool.install_hint), gray));
                }
            }
        }

        let start_y = b.y + b.h.saturating_sub(lines.len() as u16) / 2;
        for row in 0..b.h {
            let y = b.y + row;
            let line_i = (y as i32) - (start_y as i32);
            if line_i >= 0 && (line_i as usize) < lines.len() {
                let (ref text, style) = lines[line_i as usize];
                let pad_left = b.w.saturating_sub(display_width(text, 1)) / 2;
                let left_spaces: String = " ".repeat(pad_left as usize);
                let centered = format!("{}{}", left_spaces, text);
                surface.print_line(b.x, y, &centered, b.w, style);
            } else {
                surface.print_line(b.x, y, "", b.w, Style::default());
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Lazy detection on first event (draw is &self, so we do it here)
        self.ensure_tools();

        if let Event::Key(key) = event {
            if key.code == KeyCode::Char(':') {
                queue.put_command(crate::commands::CM_COMMAND_MODE, None);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
