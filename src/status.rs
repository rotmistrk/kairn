//! KairnStatusBar — status bar with Normal mode (key labels) and Prompt mode (M-x input).

use txv_core::prelude::*;
use txv_widgets::{InputLine, StatusBar};

use crate::commands::*;

/// Status bar mode.
enum Mode {
    Normal,
    Prompt,
}

/// Application status bar with command mode support.
pub struct KairnStatusBar {
    inner: StatusBar,
    input: InputLine,
    mode: Mode,
    completer: Option<Box<dyn Completer>>,
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
                code: KeyCode::Char('x'),
                modifiers: KeyMod { ctrl: false, alt: true, shift: false },
            },
            CM_COMMAND_MODE,
            "M-x",
        );
        // macOS sends ≈ for Alt-x in some terminal configs
        bar.add_item(
            KeyEvent {
                code: KeyCode::Char('≈'),
                modifiers: KeyMod::default(),
            },
            CM_COMMAND_MODE,
            "",
        );
        bar.add_item(
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyMod { ctrl: true, alt: false, shift: false },
            },
            CM_QUIT,
            "^Q:Quit",
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
                // Store display strings for potential popup (future)
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
        self.input.set_bounds(r);
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
            Mode::Prompt => self.input.needs_redraw(),
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
            Mode::Normal => self.inner.draw(surface),
            Mode::Prompt => {
                let b = self.inner.bounds();
                let style = Style {
                    attrs: Attrs { reverse: true, ..Attrs::default() },
                    ..Style::default()
                };
                // Draw prompt prefix
                surface.hline(b.x, b.y, b.w, ' ', style);
                surface.print(b.x, b.y, ":", style);
                // Draw input after the ":"
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
                // Check for CM_COMMAND_MODE command to enter prompt
                if let Event::Command { id, .. } = event {
                    if *id == CM_COMMAND_MODE {
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
                            // Extract command text and emit CM_EXECUTE_COMMAND
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
