//! Scrollable list with single selection.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use txv::surface::Surface;

use crate::scroll_view::ScrollView;
use crate::widget::{EventResult, Widget, WidgetAction};

/// Data source for a list view.
pub trait ListData {
    /// Number of items.
    fn len(&self) -> usize;

    /// Whether the list is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Render a single item into a one-row surface.
    fn render_item(&self, index: usize, surface: &mut Surface<'_>, selected: bool);
}

/// Scrollable list with keyboard navigation and selection.
pub struct ListView<D: ListData> {
    data: D,
    selected: usize,
    scroll: ScrollView,
}

impl<D: ListData> ListView<D> {
    /// Create a new list view with the given data source.
    pub fn new(data: D) -> Self {
        let mut scroll = ScrollView::new();
        scroll.set_content_size(data.len(), 0);
        Self {
            data,
            selected: 0,
            scroll,
        }
    }

    /// Get the selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected index.
    pub fn set_selected(&mut self, index: usize) {
        if self.data.is_empty() {
            return;
        }
        self.selected = index.min(self.data.len().saturating_sub(1));
    }

    /// Replace the data source. Resets selection to 0.
    pub fn set_data(&mut self, data: D) {
        self.scroll.set_content_size(data.len(), 0);
        self.data = data;
        self.selected = 0;
        self.scroll.scroll_to_top();
    }

    /// Get a reference to the data source.
    pub fn data(&self) -> &D {
        &self.data
    }

    fn move_up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);
    }

    fn move_down(&mut self, amount: usize) {
        if self.data.is_empty() {
            return;
        }
        let max = self.data.len().saturating_sub(1);
        self.selected = (self.selected + amount).min(max);
    }
}

impl<D: ListData> Widget for ListView<D> {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let h = surface.height();
        let w = surface.width();
        let range = self.scroll.visible_range(h);

        for (row_idx, item_idx) in range.enumerate() {
            let mut row_surface = surface.sub(0, row_idx as u16, w, 1);
            self.data
                .render_item(item_idx, &mut row_surface, item_idx == self.selected);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if self.data.is_empty() {
            return match key.code {
                KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
                _ => EventResult::Ignored,
            };
        }
        let _ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Up => {
                self.move_up(1);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.move_down(1);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                self.move_up(10);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                self.move_down(10);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.selected = 0;
                self.scroll.ensure_visible(0, 0);
                EventResult::Consumed
            }
            KeyCode::End => {
                self.selected = self.data.len().saturating_sub(1);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Enter => {
                EventResult::Action(WidgetAction::Selected(self.selected.to_string()))
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
    use txv::cell::{ColorMode, Style};
    use txv::screen::Screen;

    struct TestData {
        items: Vec<String>,
    }

    impl ListData for TestData {
        fn len(&self) -> usize {
            self.items.len()
        }

        fn render_item(&self, index: usize, surface: &mut Surface<'_>, selected: bool) {
            let style = if selected {
                Style {
                    attrs: txv::cell::Attrs {
                        reverse: true,
                        ..txv::cell::Attrs::default()
                    },
                    ..Style::default()
                }
            } else {
                Style::default()
            };
            surface.print(0, 0, &self.items[index], style);
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_list(n: usize) -> ListView<TestData> {
        let items: Vec<String> = (0..n).map(|i| format!("item{i}")).collect();
        ListView::new(TestData { items })
    }

    #[test]
    fn new_selects_first() {
        let lv = make_list(5);
        assert_eq!(lv.selected(), 0);
    }

    #[test]
    fn move_down_and_up() {
        let mut lv = make_list(5);
        lv.handle_key(key(KeyCode::Down));
        assert_eq!(lv.selected(), 1);
        lv.handle_key(key(KeyCode::Down));
        assert_eq!(lv.selected(), 2);
        lv.handle_key(key(KeyCode::Up));
        assert_eq!(lv.selected(), 1);
    }

    #[test]
    fn clamp_at_bounds() {
        let mut lv = make_list(3);
        lv.handle_key(key(KeyCode::Up));
        assert_eq!(lv.selected(), 0);
        lv.handle_key(key(KeyCode::End));
        assert_eq!(lv.selected(), 2);
        lv.handle_key(key(KeyCode::Down));
        assert_eq!(lv.selected(), 2);
    }

    #[test]
    fn home_end() {
        let mut lv = make_list(10);
        lv.handle_key(key(KeyCode::End));
        assert_eq!(lv.selected(), 9);
        lv.handle_key(key(KeyCode::Home));
        assert_eq!(lv.selected(), 0);
    }

    #[test]
    fn page_up_down() {
        let mut lv = make_list(30);
        lv.handle_key(key(KeyCode::PageDown));
        assert_eq!(lv.selected(), 10);
        lv.handle_key(key(KeyCode::PageUp));
        assert_eq!(lv.selected(), 0);
    }

    #[test]
    fn enter_selects() {
        let mut lv = make_list(5);
        lv.handle_key(key(KeyCode::Down));
        let result = lv.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Selected(s)) if s == "1"
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut lv = make_list(5);
        let result = lv.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn empty_list_handling() {
        let mut lv = make_list(0);
        let result = lv.handle_key(key(KeyCode::Down));
        assert!(matches!(result, EventResult::Ignored));
        let result = lv.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn set_selected_clamps() {
        let mut lv = make_list(5);
        lv.set_selected(100);
        assert_eq!(lv.selected(), 4);
    }

    #[test]
    fn set_data_resets() {
        let mut lv = make_list(5);
        lv.set_selected(3);
        lv.set_data(TestData {
            items: vec!["a".into(), "b".into()],
        });
        assert_eq!(lv.selected(), 0);
        assert_eq!(lv.data().len(), 2);
    }

    #[test]
    fn render_items() {
        let lv = make_list(3);
        let mut screen = Screen::with_color_mode(20, 5, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            lv.render(&mut s, true);
        }
        let text = screen.to_text();
        assert!(text.contains("item0"));
        assert!(text.contains("item1"));
        assert!(text.contains("item2"));
    }

    #[test]
    fn render_selected_has_reverse() {
        let lv = make_list(3);
        let mut screen = Screen::with_color_mode(20, 5, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            lv.render(&mut s, true);
        }
        // First item (selected) should have reverse attr
        assert!(screen.cell(0, 0).style.attrs.reverse);
        // Second item should not
        assert!(!screen.cell(0, 1).style.attrs.reverse);
    }
}
