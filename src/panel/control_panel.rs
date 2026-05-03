//! Control panel: symbol outline, diagnostics (stub for LSP Phase C).

use crossterm::event::KeyEvent;
use txv::cell::Style;
use txv::surface::Surface;
use txv_widgets::{EventResult, Widget};

/// Which mode the control panel is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlMode {
    #[default]
    Outline,
    Blame,
    Diagnostics,
}

/// Control panel: outline, blame, diagnostics.
pub struct ControlPanel {
    mode: ControlMode,
}

impl ControlPanel {
    /// Create a new control panel.
    pub fn new() -> Self {
        Self {
            mode: ControlMode::Outline,
        }
    }

    /// Current mode.
    pub fn mode(&self) -> ControlMode {
        self.mode
    }

    /// Cycle mode.
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            ControlMode::Outline => ControlMode::Blame,
            ControlMode::Blame => ControlMode::Diagnostics,
            ControlMode::Diagnostics => ControlMode::Outline,
        };
    }
}

impl Widget for ControlPanel {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let label = match self.mode {
            ControlMode::Outline => "Outline (LSP Phase C)",
            ControlMode::Blame => "Blame",
            ControlMode::Diagnostics => "Diagnostics",
        };
        let dim = Style {
            fg: txv::cell::Color::Palette(243),
            ..Style::default()
        };
        surface.print(1, 0, label, dim);
    }

    fn handle_key(&mut self, _key: KeyEvent) -> EventResult {
        EventResult::Ignored
    }

    fn focusable(&self) -> bool {
        true
    }
}
