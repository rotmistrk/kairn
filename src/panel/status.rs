//! Application status bar with optional prompt overlay.

use crossterm::event::KeyEvent;
use txv::cell::{Color, Style};
use txv::surface::Surface;
use txv_widgets::{EventResult, InputLine, StatusBar, StatusSpan, Widget};

/// Status bar wrapper with kairn-specific state.
pub struct AppStatusBar {
    bar: StatusBar,
    prompt: Option<InputLine>,
}

impl AppStatusBar {
    /// Create a new status bar.
    pub fn new() -> Self {
        let mut bar = StatusBar::new();
        let bg = Style {
            fg: Color::Rgb(235, 219, 178),
            bg: Color::Palette(239),
            ..Style::default()
        };
        bar.set_bg(bg);
        Self { bar, prompt: None }
    }

    /// Update the left status spans.
    pub fn set_left(&mut self, spans: Vec<StatusSpan>) {
        self.bar.set_left(spans);
    }

    /// Update the right status spans.
    pub fn set_right(&mut self, spans: Vec<StatusSpan>) {
        self.bar.set_right(spans);
    }

    /// Activate a prompt (ex-command or search).
    pub fn activate_prompt(&mut self, prefix: &str) {
        self.prompt = Some(InputLine::new(prefix));
    }

    /// Whether a prompt is active.
    pub fn has_prompt(&self) -> bool {
        self.prompt.is_some()
    }

    /// Get the prompt text (if active).
    pub fn prompt_text(&self) -> Option<&str> {
        self.prompt.as_ref().map(|p| p.text())
    }

    /// Dismiss the prompt.
    pub fn dismiss_prompt(&mut self) {
        self.prompt = None;
    }
}

impl Widget for AppStatusBar {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        if let Some(ref prompt) = self.prompt {
            prompt.render(surface, focused);
        } else {
            self.bar.render(surface, false);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if let Some(ref mut prompt) = self.prompt {
            prompt.handle_key(key)
        } else {
            EventResult::Ignored
        }
    }

    fn focusable(&self) -> bool {
        self.prompt.is_some()
    }
}
