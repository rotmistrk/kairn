//! WelcomeView — shown when center slot has no tabs.

use txv_core::prelude::*;

pub struct WelcomeView {
    state: ViewState,
}

impl WelcomeView {
    pub fn new() -> Self {
        Self { state: ViewState::default() }
    }
}

impl View for WelcomeView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str { "Welcome" }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 { return; }
        let dim = Style { fg: Color::Ansi(8), ..Style::default() };
        let bright = Style { fg: Color::Ansi(14), ..Style::default() };
        let lines: &[(&str, Style)] = &[
            ("╦╔═╔═╗╦╦═╗╔╗╔", bright),
            ("╠╩╗╠═╣║╠╦╝║║║", bright),
            ("╩ ╩╩ ╩╩╩╚═╝╚╝", bright),
            ("", dim),
            ("F1:Help  F2:Tree  F3:Main  F4:Term", dim),
            ("Ctrl-Q:Quit  M-x:Command", dim),
        ];
        let start_y = b.y + b.h.saturating_sub(lines.len() as u16) / 2;
        for row in 0..b.h {
            let y = b.y + row;
            let line_i = (y as i32) - (start_y as i32);
            if line_i >= 0 && (line_i as usize) < lines.len() {
                let (text, style) = lines[line_i as usize];
                let pad_left = b.w.saturating_sub(text.len() as u16) / 2;
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
