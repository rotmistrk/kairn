//! ProgressBar — determinate or indeterminate progress indicator.

use txv_core::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProgressMode {
    Determinate,
    Indeterminate,
}

pub struct ProgressBar {
    state: ViewState,
    pub mode: ProgressMode,
    pub progress: f32, // 0.0..=1.0
    pub tick: u16,     // for indeterminate animation
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: false,
                ..ViewOptions::default()
            }),
            mode: ProgressMode::Determinate,
            progress: 0.0,
            tick: 0,
        }
    }

    pub fn set_progress(&mut self, p: f32) {
        self.progress = p.clamp(0.0, 1.0);
        self.state.mark_dirty();
    }

    pub fn advance_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.state.mark_dirty();
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ProgressBar {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let filled_style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let empty_style = Style::default();

        match self.mode {
            ProgressMode::Determinate => {
                let filled = (self.progress * b.w as f32) as u16;
                for col in 0..b.w {
                    let style = if col < filled {
                        filled_style
                    } else {
                        empty_style
                    };
                    surface.put(b.x + col, b.y, '░', style);
                }
            }
            ProgressMode::Indeterminate => {
                let pos = self.tick % b.w;
                let width = 3.min(b.w);
                for col in 0..b.w {
                    let in_bar = col >= pos && col < pos + width;
                    let style = if in_bar {
                        filled_style
                    } else {
                        empty_style
                    };
                    surface.put(b.x + col, b.y, '░', style);
                }
            }
        }
    }

    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        if let Event::Tick = event {
            if self.mode == ProgressMode::Indeterminate {
                self.advance_tick();
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
