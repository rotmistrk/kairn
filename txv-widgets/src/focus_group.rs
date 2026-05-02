//! Focus group — manages Tab/Shift-Tab cycling over a widget list.

use crossterm::event::{KeyCode, KeyEvent};
use txv::surface::Surface;

use crate::widget::{EventResult, Widget, WidgetAction};

/// Manages focus cycling across a collection of widgets.
pub struct FocusGroup {
    widgets: Vec<Box<dyn Widget>>,
    focused: usize,
}

impl FocusGroup {
    /// Create an empty focus group.
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            focused: 0,
        }
    }

    /// Add a widget to the group.
    pub fn add(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    /// Number of widgets.
    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    /// Whether the group is empty.
    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    /// Index of the currently focused widget.
    pub fn focused(&self) -> usize {
        self.focused
    }

    /// Set focus to a specific index.
    pub fn set_focused(&mut self, index: usize) {
        if index < self.widgets.len() {
            self.focused = index;
        }
    }

    /// Get a reference to a widget by index.
    pub fn get(&self, index: usize) -> Option<&dyn Widget> {
        self.widgets.get(index).map(|w| w.as_ref())
    }

    /// Get a mutable reference to a widget by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut (dyn Widget + 'static)> {
        self.widgets.get_mut(index).map(|w| &mut **w)
    }

    /// Move focus to the next focusable widget.
    pub fn focus_next(&mut self) {
        if self.widgets.is_empty() {
            return;
        }
        let start = self.focused;
        loop {
            self.focused = (self.focused + 1) % self.widgets.len();
            if self.widgets[self.focused].focusable() || self.focused == start {
                break;
            }
        }
    }

    /// Move focus to the previous focusable widget.
    pub fn focus_prev(&mut self) {
        if self.widgets.is_empty() {
            return;
        }
        let start = self.focused;
        loop {
            self.focused = if self.focused == 0 {
                self.widgets.len() - 1
            } else {
                self.focused - 1
            };
            if self.widgets[self.focused].focusable() || self.focused == start {
                break;
            }
        }
    }

    /// Render all widgets using a layout callback.
    /// The callback receives `(index, widget, focused)` and a surface
    /// to render into. Caller is responsible for providing sub-surfaces.
    pub fn render_with<F>(&self, surface: &mut Surface<'_>, mut layout_fn: F)
    where
        F: FnMut(usize, &dyn Widget, bool, &mut Surface<'_>),
    {
        for (i, widget) in self.widgets.iter().enumerate() {
            layout_fn(i, widget.as_ref(), i == self.focused, surface);
        }
    }
}

impl Default for FocusGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for FocusGroup {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        // Default: stack widgets vertically, one row each
        let w = surface.width();
        for (i, widget) in self.widgets.iter().enumerate() {
            if i as u16 >= surface.height() {
                break;
            }
            let mut row = surface.sub(0, i as u16, w, 1);
            widget.render(&mut row, focused && i == self.focused);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        // Tab/BackTab cycle focus
        match key.code {
            KeyCode::Tab => {
                self.focus_next();
                return EventResult::Consumed;
            }
            KeyCode::BackTab => {
                self.focus_prev();
                return EventResult::Consumed;
            }
            _ => {}
        }

        // Delegate to focused widget
        if let Some(widget) = self.widgets.get_mut(self.focused) {
            let result = widget.handle_key(key);
            match result {
                EventResult::Action(WidgetAction::FocusNext) => {
                    self.focus_next();
                    EventResult::Consumed
                }
                EventResult::Action(WidgetAction::FocusPrev) => {
                    self.focus_prev();
                    EventResult::Consumed
                }
                other => other,
            }
        } else {
            EventResult::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use txv::cell::Style;

    struct StubWidget {
        focusable: bool,
        label: String,
    }

    impl Widget for StubWidget {
        fn render(&self, surface: &mut Surface<'_>, focused: bool) {
            let prefix = if focused { ">" } else { " " };
            let text = format!("{prefix}{}", self.label);
            surface.print(0, 0, &text, Style::default());
        }

        fn handle_key(&mut self, key: KeyEvent) -> EventResult {
            match key.code {
                KeyCode::Enter => EventResult::Action(WidgetAction::Confirmed(self.label.clone())),
                _ => EventResult::Ignored,
            }
        }

        fn focusable(&self) -> bool {
            self.focusable
        }
    }

    fn stub(label: &str) -> Box<dyn Widget> {
        Box::new(StubWidget {
            focusable: true,
            label: label.into(),
        })
    }

    fn unfocusable(label: &str) -> Box<dyn Widget> {
        Box::new(StubWidget {
            focusable: false,
            label: label.into(),
        })
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn empty_group() {
        let fg = FocusGroup::new();
        assert!(fg.is_empty());
        assert_eq!(fg.len(), 0);
        assert_eq!(fg.focused(), 0);
    }

    #[test]
    fn add_and_focus() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.add(stub("B"));
        assert_eq!(fg.len(), 2);
        assert_eq!(fg.focused(), 0);
    }

    #[test]
    fn tab_cycles_forward() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.add(stub("B"));
        fg.add(stub("C"));
        fg.handle_key(key(KeyCode::Tab));
        assert_eq!(fg.focused(), 1);
        fg.handle_key(key(KeyCode::Tab));
        assert_eq!(fg.focused(), 2);
        fg.handle_key(key(KeyCode::Tab));
        assert_eq!(fg.focused(), 0); // wraps
    }

    #[test]
    fn backtab_cycles_backward() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.add(stub("B"));
        fg.add(stub("C"));
        fg.handle_key(key(KeyCode::BackTab));
        assert_eq!(fg.focused(), 2); // wraps backward
        fg.handle_key(key(KeyCode::BackTab));
        assert_eq!(fg.focused(), 1);
    }

    #[test]
    fn skips_unfocusable() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.add(unfocusable("sep"));
        fg.add(stub("C"));
        fg.handle_key(key(KeyCode::Tab));
        assert_eq!(fg.focused(), 2); // skipped index 1
    }

    #[test]
    fn all_unfocusable_no_infinite_loop() {
        let mut fg = FocusGroup::new();
        fg.add(unfocusable("X"));
        fg.add(unfocusable("Y"));
        fg.focus_next(); // should not hang
                         // focused stays at 0 (or wherever it started)
    }

    #[test]
    fn delegates_key_to_focused() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.add(stub("B"));
        fg.set_focused(1);
        let result = fg.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "B"
        ));
    }

    #[test]
    fn set_focused_clamps() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.set_focused(10);
        assert_eq!(fg.focused(), 0); // unchanged, out of bounds
    }

    #[test]
    fn get_widget() {
        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        assert!(fg.get(0).is_some());
        assert!(fg.get(5).is_none());
    }

    #[test]
    fn render_stacks_vertically() {
        use txv::cell::ColorMode;
        use txv::screen::Screen;

        let mut fg = FocusGroup::new();
        fg.add(stub("A"));
        fg.add(stub("B"));
        let mut screen = Screen::with_color_mode(10, 5, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            fg.render(&mut s, true);
        }
        let text = screen.to_text();
        assert!(text.contains(">A")); // focused
        assert!(text.contains(" B")); // not focused
    }

    #[test]
    fn focus_next_action_handled() {
        struct FocusNextWidget;
        impl Widget for FocusNextWidget {
            fn render(&self, _s: &mut Surface<'_>, _f: bool) {}
            fn handle_key(&mut self, _k: KeyEvent) -> EventResult {
                EventResult::Action(WidgetAction::FocusNext)
            }
        }

        let mut fg = FocusGroup::new();
        fg.add(Box::new(FocusNextWidget));
        fg.add(stub("B"));
        let result = fg.handle_key(key(KeyCode::Char('x')));
        assert!(matches!(result, EventResult::Consumed));
        assert_eq!(fg.focused(), 1);
    }
}
