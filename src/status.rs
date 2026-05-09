//! KairnStatusBar — status bar with Normal mode (key labels) and Prompt mode (M-x input).

use txv_core::prelude::*;
use txv_widgets::{InputLine, StatusBar};

use crate::commands::*;

/// Status bar mode.
enum Mode {
    Normal,
    Prompt,
}

const ALT_X: KeyEvent = KeyEvent {
    code: KeyCode::Char('x'),
    modifiers: KeyMod { ctrl: false, alt: true, shift: false },
};

/// Application status bar with command mode support.
pub struct KairnStatusBar {
    inner: StatusBar,
    input: InputLine,
    mode: Mode,
    completer: Option<Box<dyn Completer>>,
}

impl Default for KairnStatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl KairnStatusBar {
    pub fn new() -> Self {
        let mut bar = StatusBar::new();
        bar.add_item(
            KeyEvent { code: KeyCode::F(1), modifiers: KeyMod::default() },
            CM_SHOW_HELP,
            "F1:Help",
        );
        bar.add_item(
            KeyEvent { code: KeyCode::F(2), modifiers: KeyMod::default() },
            CM_FOCUS_LEFT,
            "F2:Tree",
        );
        bar.add_item(
            KeyEvent { code: KeyCode::F(3), modifiers: KeyMod::default() },
            CM_FOCUS_CENTER,
            "F3:Main",
        );
        bar.add_item(
            KeyEvent { code: KeyCode::F(4), modifiers: KeyMod::default() },
            CM_FOCUS_RIGHT,
            "F4:Term",
        );
        bar.add_item(
            KeyEvent { code: KeyCode::F(5), modifiers: KeyMod::default() },
            CM_ZOOM_TOGGLE,
            "F5:Zoom",
        );
        bar.add_item(
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyMod { ctrl: true, alt: false, shift: false },
            },
            CM_QUIT,
            "^Q:Quit",
        );
        // M-x via macOS Option-x (sends ≈)
        bar.add_item(
            KeyEvent {
                code: KeyCode::Char('≈'),
                modifiers: KeyMod::default(),
            },
            CM_COMMAND_MODE,
            "",
        );
        // Ctrl-Shift arrows
        bar.add_item(
            KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyMod { ctrl: true, alt: false, shift: true },
            },
            CM_TAB_PREV,
            "",
        );
        bar.add_item(
            KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyMod { ctrl: true, alt: false, shift: true },
            },
            CM_TAB_NEXT,
            "",
        );
        bar.add_item(
            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyMod { ctrl: true, alt: false, shift: true },
            },
            CM_FOCUS_PREV,
            "",
        );
        bar.add_item(
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyMod { ctrl: true, alt: false, shift: true },
            },
            CM_FOCUS_NEXT,
            "",
        );
        Self {
            inner: bar,
            input: InputLine::new(),
            mode: Mode::Normal,
            completer: None,
        }
    }

    pub fn set_context(&mut self, ctx: impl Into<String>) {
        self.inner.set_context(ctx);
    }

    pub fn set_completer(&mut self, completer: Box<dyn Completer>) {
        self.completer = Some(completer);
    }

    fn enter_prompt(&mut self) {
        self.mode = Mode::Prompt;
        self.input.clear();
        // Position input after the ":" prefix
        let b = self.inner.bounds();
        if b.w > 1 {
            self.input.set_bounds(Rect::new(b.x + 1, b.y, b.w - 1, 1));
        }
    }

    fn exit_prompt(&mut self) {
        self.mode = Mode::Normal;
        self.input.clear();
    }

    fn try_complete(&mut self) {
        if let Some(ref completer) = self.completer {
            let completions = completer.complete(&self.input.text, self.input.cursor);
            if completions.len() == 1 {
                self.input.set_text(&completions[0].text);
            } else if !completions.is_empty() {
                self.input.completions = completions.iter()
                    .map(|c| c.display.clone())
                    .collect();
            }
        }
    }
}

impl View for KairnStatusBar {
    fn bounds(&self) -> Rect { self.inner.bounds() }
    fn set_bounds(&mut self, r: Rect) {
        self.inner.set_bounds(r);
        // Input gets bounds minus the ":" prefix
        if r.w > 1 {
            self.input.set_bounds(Rect::new(r.x + 1, r.y, r.w - 1, 1));
        }
    }
    fn options(&self) -> ViewOptions {
        ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        }
    }
    fn title(&self) -> &str { "" }
    fn needs_redraw(&self) -> bool {
        match self.mode {
            Mode::Normal => self.inner.needs_redraw(),
            Mode::Prompt => true, // always redraw in prompt mode
        }
    }
    fn mark_redrawn(&mut self) {
        self.inner.mark_redrawn();
        self.input.mark_redrawn();
    }
    fn select(&mut self) {}
    fn unselect(&mut self) {}

    fn draw(&self, surface: &mut Surface) {
        match self.mode {
            Mode::Normal => {
                self.inner.draw(surface);
                // Also draw "M-x" label (inner doesn't have it as a StatusItem)
                let b = self.inner.bounds();
                let style = Style {
                    attrs: Attrs { reverse: true, ..Attrs::default() },
                    ..Style::default()
                };
                // Find space after last item for M-x label
                let mx_text = " M-x ";
                let items_len: u16 = self.inner.items.iter()
                    .map(|i| i.label.len() as u16 + 2)
                    .sum();
                if items_len + mx_text.len() as u16 <= b.w {
                    surface.print(b.x + items_len, b.y, mx_text, style);
                }
            }
            Mode::Prompt => {
                let b = self.inner.bounds();
                let style = Style {
                    attrs: Attrs { reverse: true, ..Attrs::default() },
                    ..Style::default()
                };
                surface.hline(b.x, b.y, b.w, ' ', style);
                surface.print(b.x, b.y, ":", style);
                self.input.draw(surface);
            }
        }
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        match self.mode {
            Mode::Normal => {
                // Intercept Alt-x directly to enter prompt mode
                if let Event::Key(key) = event {
                    if *key == ALT_X {
                        self.enter_prompt();
                        return HandleResult::Consumed;
                    }
                    // macOS Option-x sends ≈
                    if key.code == KeyCode::Char('≈') && !key.modifiers.ctrl && !key.modifiers.alt {
                        self.enter_prompt();
                        return HandleResult::Consumed;
                    }
                }
                // Normal key→command translation
                self.inner.handle(event, queue)
            }
            Mode::Prompt => {
                let Event::Key(key) = event else {
                    return HandleResult::Ignored;
                };
                // Tab triggers completion
                if key.code == KeyCode::Tab {
                    self.try_complete();
                    return HandleResult::Consumed;
                }
                // Forward to InputLine
                let result = self.input.handle(event, queue);
                // Check if InputLine emitted CM_OK (Enter) or CM_CANCEL (Esc)
                let events = queue.drain();
                for ev in events {
                    if let Event::Command { id, data } = &ev {
                        if *id == CM_OK {
                            if let Some(boxed) = data.as_ref() {
                                if let Some(text) = boxed.downcast_ref::<String>() {
                                    let cmd_text = text.clone();
                                    self.exit_prompt();
                                    queue.put_command(
                                        CM_EXECUTE_COMMAND,
                                        Some(Box::new(cmd_text)),
                                    );
                                    return HandleResult::Consumed;
                                }
                            }
                        } else if *id == CM_CANCEL {
                            self.exit_prompt();
                            return HandleResult::Consumed;
                        }
                    }
                    queue.put(ev);
                }
                result
            }
        }
    }
}
