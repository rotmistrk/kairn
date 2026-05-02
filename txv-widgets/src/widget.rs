//! Core widget trait and types.

use crossterm::event::KeyEvent;

/// Result of widget input handling.
pub enum EventResult {
    /// Widget consumed the event.
    Consumed,
    /// Widget did not handle the event.
    Ignored,
    /// Widget produced an action.
    Action(WidgetAction),
}

/// Actions widgets can produce.
pub enum WidgetAction {
    /// Item selected (index or value).
    Selected(String),
    /// User confirmed (Enter).
    Confirmed(String),
    /// User cancelled (Esc).
    Cancelled,
    /// Close requested.
    Close,
    /// Move focus to next widget.
    FocusNext,
    /// Move focus to previous widget.
    FocusPrev,
    /// Custom action.
    Custom(Box<dyn std::any::Any + Send>),
}

/// An interactive component that can render and handle input.
pub trait Widget {
    /// Render this widget to a surface.
    fn render(&self, surface: &mut txv::surface::Surface<'_>, focused: bool);

    /// Handle a key event.
    fn handle_key(&mut self, key: KeyEvent) -> EventResult;

    /// Whether this widget can receive focus.
    fn focusable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_result_consumed() {
        let r = EventResult::Consumed;
        assert!(matches!(r, EventResult::Consumed));
    }

    #[test]
    fn event_result_ignored() {
        let r = EventResult::Ignored;
        assert!(matches!(r, EventResult::Ignored));
    }

    #[test]
    fn event_result_action() {
        let r = EventResult::Action(WidgetAction::Cancelled);
        assert!(matches!(r, EventResult::Action(WidgetAction::Cancelled)));
    }

    #[test]
    fn widget_action_selected() {
        let a = WidgetAction::Selected("item".into());
        assert!(matches!(a, WidgetAction::Selected(s) if s == "item"));
    }

    #[test]
    fn widget_action_confirmed() {
        let a = WidgetAction::Confirmed("ok".into());
        assert!(matches!(a, WidgetAction::Confirmed(s) if s == "ok"));
    }

    #[test]
    fn widget_action_close() {
        let a = WidgetAction::Close;
        assert!(matches!(a, WidgetAction::Close));
    }

    #[test]
    fn widget_action_focus_next() {
        let a = WidgetAction::FocusNext;
        assert!(matches!(a, WidgetAction::FocusNext));
    }

    #[test]
    fn widget_action_focus_prev() {
        let a = WidgetAction::FocusPrev;
        assert!(matches!(a, WidgetAction::FocusPrev));
    }

    #[test]
    fn widget_action_custom() {
        let a = WidgetAction::Custom(Box::new(42u32));
        assert!(matches!(a, WidgetAction::Custom(_)));
    }
}
