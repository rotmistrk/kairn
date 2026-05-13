//! WelcomeView — shown when center slot has no tabs.

use txv_core::prelude::*;

pub struct WelcomeView {
    state: ViewState,
}

impl Default for WelcomeView {
    fn default() -> Self {
        Self::new()
    }
}

impl WelcomeView {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
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
        let lines: &[(&str, Style)] = &[
            ("╦╔═╔═╗╦╦═╗╔╗╔", bright),
            ("╠╩╗╠═╣║╠╦╝║║║", bright),
            ("╩ ╩╩ ╩╩╩╚═╝╚╝", bright),
            ("", dim),
            ("Navigate files in tree (F2), Enter to open", dim),
            (":e <file>  to open by name", dim),
            (":q  close tab   :w  save", dim),
            ("", dim),
            ("F1:Help  F5:Zoom  M-x:Command  Ctrl-Q:Quit", dim),
        ];
        let start_y = b.y + b.h.saturating_sub(lines.len() as u16) / 2;
        for row in 0..b.h {
            let y = b.y + row;
            let line_i = (y as i32) - (start_y as i32);
            if line_i >= 0 && (line_i as usize) < lines.len() {
                let (text, style) = lines[line_i as usize];
                let pad_left = b.w.saturating_sub(display_width(text, 1)) / 2;
                // Build centered line
                let left_spaces: String = " ".repeat(pad_left as usize);
                let centered = format!("{}{}", left_spaces, text);
                surface.print_line(b.x, y, &centered, b.w, style);
            } else {
                surface.print_line(b.x, y, "", b.w, Style::default());
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char(':') {
                queue.put_command(crate::commands::CM_COMMAND_MODE, None);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
