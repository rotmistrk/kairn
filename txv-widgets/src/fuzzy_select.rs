//! Fuzzy select — input line + filtered list combo.

use crossterm::event::{KeyCode, KeyEvent};
use txv::cell::Style;
use txv::surface::Surface;

use crate::input_line::InputLine;
use crate::widget::{EventResult, Widget, WidgetAction};

/// Input + filtered list combo for fuzzy selection.
pub struct FuzzySelect {
    input: InputLine,
    items: Vec<String>,
    filtered: Vec<(usize, i64)>, // (original_index, score)
    selected: usize,
    max_visible: usize,
    /// Style for the selected item.
    pub selected_style: Style,
    /// Style for unselected items.
    pub item_style: Style,
}

impl FuzzySelect {
    /// Create a new fuzzy select with the given items.
    pub fn new(items: Vec<String>) -> Self {
        let filtered: Vec<(usize, i64)> = items.iter().enumerate().map(|(i, _)| (i, 0)).collect();
        Self {
            input: InputLine::new(""),
            items,
            filtered,
            selected: 0,
            max_visible: 20,
            selected_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            item_style: Style::default(),
        }
    }

    /// Get the currently selected item, if any.
    pub fn selected_item(&self) -> Option<&str> {
        self.filtered
            .get(self.selected)
            .map(|(idx, _)| self.items[*idx].as_str())
    }

    /// Replace the item list and re-filter.
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.refilter();
    }

    /// Set the maximum number of visible items.
    pub fn set_max_visible(&mut self, n: usize) {
        self.max_visible = n;
    }

    fn refilter(&mut self) {
        let query = self.input.text().to_lowercase();
        self.filtered.clear();
        if query.is_empty() {
            self.filtered = self.items.iter().enumerate().map(|(i, _)| (i, 0)).collect();
        } else {
            for (i, item) in self.items.iter().enumerate() {
                let lower = item.to_lowercase();
                if let Some(pos) = lower.find(&query) {
                    // Score: prefer earlier matches and shorter items
                    let score = -(pos as i64) - (item.len() as i64);
                    self.filtered.push((i, score));
                }
            }
            self.filtered.sort_by(|a, b| b.1.cmp(&a.1));
        }
        self.selected = 0;
    }

    fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self) {
        if !self.filtered.is_empty() {
            let max = self.filtered.len().saturating_sub(1);
            self.selected = (self.selected + 1).min(max);
        }
    }
}

impl Widget for FuzzySelect {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        let w = surface.width();
        let h = surface.height();
        if h == 0 {
            return;
        }

        // Row 0: input line
        let mut input_surface = surface.sub(0, 0, w, 1);
        self.input.render(&mut input_surface, focused);

        // Remaining rows: filtered items
        let list_h = (h.saturating_sub(1) as usize).min(self.max_visible);
        let scroll_offset = if self.selected >= list_h {
            self.selected - list_h + 1
        } else {
            0
        };

        for row in 0..list_h {
            let idx = scroll_offset + row;
            if idx >= self.filtered.len() {
                break;
            }
            let (orig_idx, _) = self.filtered[idx];
            let style = if idx == self.selected {
                self.selected_style
            } else {
                self.item_style
            };
            let mut row_surface = surface.sub(0, (row + 1) as u16, w, 1);
            row_surface.hline(0, 0, w, ' ', style);
            row_surface.print(0, 0, &self.items[orig_idx], style);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        match key.code {
            KeyCode::Up => {
                self.move_up();
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.move_down();
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(item) = self.selected_item() {
                    EventResult::Action(WidgetAction::Confirmed(item.to_string()))
                } else {
                    EventResult::Action(WidgetAction::Cancelled)
                }
            }
            KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
            _ => {
                let result = self.input.handle_key(key);
                // Re-filter after any input change
                if matches!(result, EventResult::Consumed) {
                    self.refilter();
                }
                result
            }
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

    fn make_select() -> FuzzySelect {
        FuzzySelect::new(vec![
            "apple".into(),
            "banana".into(),
            "cherry".into(),
            "apricot".into(),
            "blueberry".into(),
        ])
    }

    #[test]
    fn new_shows_all_items() {
        let fs = make_select();
        assert_eq!(fs.filtered.len(), 5);
        assert_eq!(fs.selected_item(), Some("apple"));
    }

    #[test]
    fn typing_filters() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Char('a')));
        fs.handle_key(key(KeyCode::Char('p')));
        // "ap" matches: apple, apricot
        assert_eq!(fs.filtered.len(), 2);
        assert!(fs.selected_item().is_some());
    }

    #[test]
    fn case_insensitive_filter() {
        let mut fs = FuzzySelect::new(vec!["Apple".into(), "BANANA".into()]);
        fs.handle_key(key(KeyCode::Char('a')));
        // "a" matches "Apple" and "BANANA"
        assert_eq!(fs.filtered.len(), 2);
    }

    #[test]
    fn no_matches() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Char('z')));
        fs.handle_key(key(KeyCode::Char('z')));
        assert!(fs.filtered.is_empty());
        assert!(fs.selected_item().is_none());
    }

    #[test]
    fn up_down_navigation() {
        let mut fs = make_select();
        assert_eq!(fs.selected, 0);
        fs.handle_key(key(KeyCode::Down));
        assert_eq!(fs.selected, 1);
        fs.handle_key(key(KeyCode::Down));
        assert_eq!(fs.selected, 2);
        fs.handle_key(key(KeyCode::Up));
        assert_eq!(fs.selected, 1);
    }

    #[test]
    fn up_clamps_at_zero() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Up));
        assert_eq!(fs.selected, 0);
    }

    #[test]
    fn down_clamps_at_end() {
        let mut fs = make_select();
        for _ in 0..10 {
            fs.handle_key(key(KeyCode::Down));
        }
        assert_eq!(fs.selected, 4);
    }

    #[test]
    fn enter_confirms_selected() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Down)); // banana
        let result = fs.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "banana"
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut fs = make_select();
        let result = fs.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn enter_on_empty_cancels() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Char('z')));
        fs.handle_key(key(KeyCode::Char('z')));
        let result = fs.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn set_items_refilters() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Char('x')));
        assert!(fs.filtered.is_empty());
        fs.set_items(vec!["fox".into(), "box".into()]);
        // "x" matches both
        assert_eq!(fs.filtered.len(), 2);
    }

    #[test]
    fn render_shows_input_and_items() {
        let fs = make_select();
        let mut screen = Screen::with_color_mode(30, 10, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            fs.render(&mut s, true);
        }
        let text = screen.to_text();
        assert!(text.contains("apple"));
        assert!(text.contains("banana"));
    }

    #[test]
    fn backspace_refilters() {
        let mut fs = make_select();
        fs.handle_key(key(KeyCode::Char('a')));
        fs.handle_key(key(KeyCode::Char('p')));
        let count_after_ap = fs.filtered.len();
        fs.handle_key(key(KeyCode::Backspace));
        // After removing 'p', filter is just "a" — should match more
        assert!(fs.filtered.len() >= count_after_ap);
    }
}
