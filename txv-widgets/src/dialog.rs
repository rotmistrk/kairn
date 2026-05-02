//! Modal dialog with title, message, and buttons.

use crossterm::event::{KeyCode, KeyEvent};
use txv::border::{draw_border, BorderMode, BorderStyle};
use txv::cell::Style;
use txv::layout::Rect;
use txv::surface::Surface;
use txv::text::wrap;

use crate::input_line::InputLine;
use crate::widget::{EventResult, Widget, WidgetAction};

/// The kind of dialog.
pub enum DialogKind {
    /// Informational message with OK button.
    Info,
    /// Yes/No confirmation.
    Confirm,
    /// Text input prompt.
    Prompt {
        /// Default value for the input.
        default: String,
    },
}

/// A modal dialog with title, message, and buttons.
pub struct Dialog {
    title: String,
    message: String,
    kind: DialogKind,
    input: Option<InputLine>,
    selected_button: usize,
    button_labels: Vec<String>,
    /// Border style for the dialog.
    pub border_style: BorderStyle,
    /// Style for the message text.
    pub text_style: Style,
    /// Style for the selected button.
    pub button_active_style: Style,
    /// Style for unselected buttons.
    pub button_inactive_style: Style,
}

impl Dialog {
    /// Create an info dialog with an OK button.
    pub fn info(title: &str, message: &str) -> Self {
        Self::new(title, message, DialogKind::Info, vec!["OK".into()])
    }

    /// Create a confirmation dialog with Yes/No buttons.
    pub fn confirm(title: &str, message: &str) -> Self {
        Self::new(
            title,
            message,
            DialogKind::Confirm,
            vec!["Yes".into(), "No".into()],
        )
    }

    /// Create a prompt dialog with an input field.
    pub fn prompt(title: &str, message: &str, default: &str) -> Self {
        let mut input = InputLine::new("");
        input.set_text(default);
        let mut dlg = Self::new(
            title,
            message,
            DialogKind::Prompt {
                default: default.to_string(),
            },
            vec!["OK".into(), "Cancel".into()],
        );
        dlg.input = Some(input);
        dlg
    }

    fn new(title: &str, message: &str, kind: DialogKind, button_labels: Vec<String>) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            kind,
            input: None,
            selected_button: 0,
            button_labels,
            border_style: BorderStyle {
                mode: BorderMode::Pretty,
                active: Style::default(),
                inactive: Style::default(),
            },
            text_style: Style::default(),
            button_active_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            button_inactive_style: Style::default(),
        }
    }

    /// Get the currently selected button label.
    pub fn selected_button_label(&self) -> &str {
        self.button_labels
            .get(self.selected_button)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    fn render_buttons(&self, surface: &mut Surface<'_>, row: u16) {
        let w = surface.width() as usize;
        let total: usize = self
            .button_labels
            .iter()
            .map(|b| b.len() + 4) // "[ label ]"
            .sum::<usize>()
            + self.button_labels.len().saturating_sub(1) * 2;
        let start = w.saturating_sub(total) / 2;
        let mut col = start as u16;
        for (i, label) in self.button_labels.iter().enumerate() {
            let style = if i == self.selected_button {
                self.button_active_style
            } else {
                self.button_inactive_style
            };
            let btn = format!("[ {label} ]");
            surface.print(col, row, &btn, style);
            col += btn.len() as u16 + 2;
        }
    }
}

impl Widget for Dialog {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        let w = surface.width();
        let h = surface.height();
        surface.fill(' ', self.text_style);

        let rect = Rect { x: 0, y: 0, w, h };
        let inner = draw_border(surface, rect, &self.title, &self.border_style, focused);
        if inner.w < 2 || inner.h < 2 {
            return;
        }

        let mut sub = surface.sub(inner.x, inner.y, inner.w, inner.h);
        let msg_width = inner.w.saturating_sub(2) as usize;
        if msg_width == 0 {
            return;
        }
        let lines = wrap(&self.message, msg_width);
        for (i, line) in lines.iter().enumerate() {
            if i as u16 >= inner.h.saturating_sub(2) {
                break;
            }
            sub.print(1, i as u16, line, self.text_style);
        }

        // Input line for prompt dialogs
        let mut button_row = (lines.len() as u16 + 1).min(inner.h.saturating_sub(1));
        if let Some(ref input) = self.input {
            if button_row < inner.h.saturating_sub(1) {
                let mut input_sub = sub.sub(1, button_row, inner.w.saturating_sub(2), 1);
                input.render(&mut input_sub, focused);
                button_row += 1;
            }
        }

        // Buttons
        if button_row < inner.h {
            self.render_buttons(&mut sub, button_row);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        // If we have an input and it's a prompt, delegate most keys to input
        if let Some(ref mut input) = self.input {
            match key.code {
                KeyCode::Tab => {
                    self.selected_button = (self.selected_button + 1) % self.button_labels.len();
                    return EventResult::Consumed;
                }
                KeyCode::BackTab => {
                    self.selected_button = if self.selected_button == 0 {
                        self.button_labels.len().saturating_sub(1)
                    } else {
                        self.selected_button - 1
                    };
                    return EventResult::Consumed;
                }
                KeyCode::Enter => {
                    if self.selected_button == 0 {
                        let text = input.text().to_string();
                        return EventResult::Action(WidgetAction::Confirmed(text));
                    }
                    return EventResult::Action(WidgetAction::Cancelled);
                }
                KeyCode::Esc => {
                    return EventResult::Action(WidgetAction::Cancelled);
                }
                _ => {
                    return input.handle_key(key);
                }
            }
        }

        match key.code {
            KeyCode::Tab | KeyCode::Right => {
                self.selected_button = (self.selected_button + 1) % self.button_labels.len();
                EventResult::Consumed
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.selected_button = if self.selected_button == 0 {
                    self.button_labels.len().saturating_sub(1)
                } else {
                    self.selected_button - 1
                };
                EventResult::Consumed
            }
            KeyCode::Enter => {
                let label = self.selected_button_label().to_string();
                match self.kind {
                    DialogKind::Info => EventResult::Action(WidgetAction::Close),
                    DialogKind::Confirm => EventResult::Action(WidgetAction::Confirmed(label)),
                    DialogKind::Prompt { .. } => {
                        EventResult::Action(WidgetAction::Confirmed(label))
                    }
                }
            }
            KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn render_dialog(dialog: &Dialog, w: u16, h: u16) -> String {
        let mut screen = Screen::with_color_mode(w, h, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            dialog.render(&mut s, true);
        }
        screen.to_text()
    }

    #[test]
    fn info_dialog_renders() {
        let dlg = Dialog::info("Alert", "Something happened");
        let text = render_dialog(&dlg, 40, 8);
        assert!(text.contains("Alert"));
        assert!(text.contains("Something happened"));
        assert!(text.contains("OK"));
    }

    #[test]
    fn confirm_dialog_renders() {
        let dlg = Dialog::confirm("Delete?", "Are you sure?");
        let text = render_dialog(&dlg, 40, 8);
        assert!(text.contains("Yes"));
        assert!(text.contains("No"));
    }

    #[test]
    fn prompt_dialog_renders() {
        let dlg = Dialog::prompt("Name", "Enter name:", "default");
        let text = render_dialog(&dlg, 40, 10);
        assert!(text.contains("Name"));
        assert!(text.contains("Enter name:"));
    }

    #[test]
    fn info_enter_closes() {
        let mut dlg = Dialog::info("Test", "msg");
        let result = dlg.handle_key(key(KeyCode::Enter));
        assert!(matches!(result, EventResult::Action(WidgetAction::Close)));
    }

    #[test]
    fn confirm_tab_switches_buttons() {
        let mut dlg = Dialog::confirm("Q", "?");
        assert_eq!(dlg.selected_button, 0);
        assert_eq!(dlg.selected_button_label(), "Yes");
        dlg.handle_key(key(KeyCode::Tab));
        assert_eq!(dlg.selected_button, 1);
        assert_eq!(dlg.selected_button_label(), "No");
        dlg.handle_key(key(KeyCode::Tab));
        assert_eq!(dlg.selected_button, 0); // wraps
    }

    #[test]
    fn confirm_enter_confirms_selected() {
        let mut dlg = Dialog::confirm("Q", "?");
        dlg.handle_key(key(KeyCode::Tab)); // select "No"
        let result = dlg.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "No"
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut dlg = Dialog::info("T", "m");
        let result = dlg.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn prompt_enter_confirms_input() {
        let mut dlg = Dialog::prompt("Name", "Enter:", "hello");
        let result = dlg.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "hello"
        ));
    }

    #[test]
    fn prompt_typing_updates_input() {
        let mut dlg = Dialog::prompt("Name", "Enter:", "");
        dlg.handle_key(key(KeyCode::Char('a')));
        dlg.handle_key(key(KeyCode::Char('b')));
        let result = dlg.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "ab"
        ));
    }

    #[test]
    fn prompt_cancel_on_second_button() {
        let mut dlg = Dialog::prompt("Name", "Enter:", "x");
        dlg.handle_key(key(KeyCode::Tab)); // select Cancel
        let result = dlg.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }
}
