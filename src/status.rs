//! StatusBarView — key-to-command translator and hint display.
//!
//! The status bar sees key events FIRST (preprocess role). When a key
//! matches a configured binding, it emits the corresponding command
//! via the outbox. It also renders key hints and context info.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use txv::cell::Style;
use txv::layout::Rect;
use txv::surface::Surface;
use txv::text::display_width;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};
use txv_widgets::CommandId;

use crate::types::CommandOutbox;

/// A key specification for matching.
#[derive(Debug, Clone)]
pub struct KeySpec {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeySpec {
    /// Check if a key event matches this spec.
    pub fn matches(&self, key: &KeyEvent) -> bool {
        self.code == key.code && self.modifiers == key.modifiers
    }
}

/// A single keybinding entry.
pub struct Binding {
    pub key: KeySpec,
    pub command: CommandId,
    pub label: String,
}

/// Status bar: translates keys to commands, renders hints.
pub struct StatusBarView {
    bindings: Vec<Binding>,
    context_text: String,
    bounds: Rect,
    pub outbox: CommandOutbox,
    style: Style,
}

impl StatusBarView {
    /// Create a status bar with the given bindings.
    pub fn new(bindings: Vec<Binding>) -> Self {
        Self {
            bindings,
            context_text: String::new(),
            bounds: Rect { x: 0, y: 0, w: 0, h: 1 },
            outbox: CommandOutbox::default(),
            style: Style {
                fg: txv::cell::Color::Ansi(0),
                bg: txv::cell::Color::Ansi(7),
                ..Style::default()
            },
        }
    }

    /// Set the right-aligned context text.
    pub fn set_context(&mut self, text: String) {
        self.context_text = text;
    }

    /// Create default keybindings for kairn per spec.
    pub fn default_bindings() -> Vec<Binding> {
        use crate::commands::*;
        vec![
            Binding {
                key: KeySpec { code: KeyCode::F(1), modifiers: KeyModifiers::NONE },
                command: CM_SHOW_HELP,
                label: "F1:Help".into(),
            },
            Binding {
                key: KeySpec { code: KeyCode::F(2), modifiers: KeyModifiers::NONE },
                command: CM_FOCUS_LEFT,
                label: "F2:Tree".into(),
            },
            Binding {
                key: KeySpec { code: KeyCode::F(3), modifiers: KeyModifiers::NONE },
                command: CM_FOCUS_CENTER,
                label: "F3:Main".into(),
            },
            Binding {
                key: KeySpec { code: KeyCode::F(4), modifiers: KeyModifiers::NONE },
                command: CM_FOCUS_RIGHT,
                label: "F4:Tools".into(),
            },
            Binding {
                key: KeySpec { code: KeyCode::F(5), modifiers: KeyModifiers::NONE },
                command: CM_ZOOM_TOGGLE,
                label: "F5:Zoom".into(),
            },
            Binding {
                key: KeySpec { code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL },
                command: CM_QUIT,
                label: "^Q:Quit".into(),
            },
        ]
    }
}

impl View for StatusBarView {
    fn draw(&self, surface: &mut Surface<'_>, _ctx: &DrawContext) {
        let w = surface.width();
        // Fill background
        surface.hline(0, 0, w, ' ', self.style);

        // Render key hints left-aligned
        let mut col: u16 = 0;
        for binding in &self.bindings {
            let text = format!(" {} ", binding.label);
            let tw = display_width(&text) as u16;
            if col + tw > w {
                break;
            }
            surface.print(col, 0, &text, self.style);
            col += tw;
        }

        // Render context text right-aligned
        if !self.context_text.is_empty() {
            let rw = display_width(&self.context_text) as u16;
            let start = w.saturating_sub(rw + 1);
            if start > col {
                surface.print(start, 0, &self.context_text, self.style);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(key) = event {
            for binding in &self.bindings {
                if binding.key.matches(key) {
                    self.outbox.emit(binding.command);
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
    }

    fn focusable(&self) -> bool {
        false
    }
}
