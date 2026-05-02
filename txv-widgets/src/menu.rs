//! Popup menu with keyboard navigation.

use crossterm::event::{KeyCode, KeyEvent};
use txv::border::{draw_border, BorderMode, BorderStyle};
use txv::cell::{Attrs, Color, Style};
use txv::layout::Rect;
use txv::surface::Surface;

use crate::scroll_view::ScrollView;
use crate::widget::{EventResult, Widget, WidgetAction};

/// A single menu item or separator.
pub struct MenuItem {
    /// Display label.
    pub label: String,
    /// Keyboard shortcut hint (display only).
    pub key_hint: String,
    /// Whether this item can be selected.
    pub enabled: bool,
    /// If true, renders as a horizontal divider line.
    pub separator: bool,
}

impl MenuItem {
    /// Create a normal enabled menu item.
    pub fn new(label: impl Into<String>, key_hint: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            key_hint: key_hint.into(),
            enabled: true,
            separator: false,
        }
    }

    /// Create a separator line.
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            key_hint: String::new(),
            enabled: false,
            separator: true,
        }
    }
}

/// Popup menu widget.
pub struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
    scroll: ScrollView,
}

impl Menu {
    /// Create a new menu. Selects the first selectable item.
    pub fn new(items: Vec<MenuItem>) -> Self {
        let mut menu = Self {
            items,
            selected: 0,
            scroll: ScrollView::new(),
        };
        menu.scroll.set_content_size(menu.items.len(), 0);
        menu.move_to_next_selectable(0, 1);
        menu
    }

    /// The currently selected item, if any.
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items
            .get(self.selected)
            .filter(|i| i.enabled && !i.separator)
    }

    /// The currently selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Computed width: max(label + key_hint) + padding + border.
    pub fn width(&self) -> u16 {
        let content_w = self
            .items
            .iter()
            .map(|i| i.label.len() + i.key_hint.len() + 4) // " label  hint "
            .max()
            .unwrap_or(10);
        (content_w + 2) as u16 // +2 for border
    }

    /// Total height including border.
    pub fn height(&self) -> u16 {
        (self.items.len() + 2) as u16 // +2 for border
    }

    fn is_selectable(&self, idx: usize) -> bool {
        self.items
            .get(idx)
            .map(|i| i.enabled && !i.separator)
            .unwrap_or(false)
    }

    fn move_to_next_selectable(&mut self, start: usize, dir: isize) {
        if self.items.is_empty() {
            return;
        }
        let len = self.items.len();
        let mut idx = start;
        for _ in 0..len {
            if self.is_selectable(idx) {
                self.selected = idx;
                return;
            }
            idx = (idx as isize + dir).rem_euclid(len as isize) as usize;
        }
    }

    fn move_down(&mut self) {
        let start = (self.selected + 1) % self.items.len();
        self.move_to_next_selectable(start, 1);
    }

    fn move_up(&mut self) {
        let len = self.items.len();
        let start = (self.selected + len - 1) % len;
        self.move_to_next_selectable(start, -1);
    }

    fn jump_to_first(&mut self) {
        self.move_to_next_selectable(0, 1);
    }

    fn jump_to_last(&mut self) {
        let last = self.items.len().saturating_sub(1);
        self.move_to_next_selectable(last, -1);
    }

    fn jump_to_letter(&mut self, ch: char) {
        let lower = ch.to_ascii_lowercase();
        let len = self.items.len();
        for offset in 1..=len {
            let idx = (self.selected + offset) % len;
            if self.is_selectable(idx) {
                let first = self.items[idx]
                    .label
                    .chars()
                    .next()
                    .map(|c| c.to_ascii_lowercase());
                if first == Some(lower) {
                    self.selected = idx;
                    return;
                }
            }
        }
    }

    fn render_item(&self, idx: usize, surface: &mut Surface<'_>, content_w: u16) {
        let item = &self.items[idx];
        let is_selected = idx == self.selected;

        if item.separator {
            let sep_style = Style {
                fg: Color::Ansi(8),
                ..Style::default()
            };
            surface.hline(0, 0, content_w, '─', sep_style);
            return;
        }

        let style = if !item.enabled {
            Style {
                fg: Color::Ansi(8),
                attrs: Attrs {
                    dim: true,
                    ..Attrs::default()
                },
                ..Style::default()
            }
        } else if is_selected {
            Style {
                fg: Color::Reset,
                bg: Color::Ansi(4),
                attrs: Attrs {
                    reverse: true,
                    ..Attrs::default()
                },
            }
        } else {
            Style::default()
        };

        if is_selected {
            surface.fill(' ', style);
        }

        let label = format!(" {}", item.label);
        surface.print(0, 0, &label, style);

        if !item.key_hint.is_empty() {
            let hint = format!("{} ", item.key_hint);
            let col = content_w.saturating_sub(hint.len() as u16);
            surface.print(col, 0, &hint, style);
        }
    }
}

impl Widget for Menu {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        let border_style = BorderStyle {
            mode: BorderMode::Pretty,
            active: Style {
                fg: Color::Reset,
                ..Style::default()
            },
            inactive: Style {
                fg: Color::Ansi(8),
                ..Style::default()
            },
        };

        let rect = Rect {
            x: 0,
            y: 0,
            w: surface.width(),
            h: surface.height(),
        };
        let inner = draw_border(surface, rect, "", &border_style, focused);

        let vh = inner.h;
        let range = self.scroll.visible_range(vh);
        for (row_idx, item_idx) in range.enumerate() {
            let mut row = surface.sub(inner.x, inner.y + row_idx as u16, inner.w, 1);
            self.render_item(item_idx, &mut row, inner.w);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if self.items.is_empty() {
            return match key.code {
                KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
                _ => EventResult::Ignored,
            };
        }
        match key.code {
            KeyCode::Up => {
                self.move_up();
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.move_down();
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.jump_to_first();
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::End => {
                self.jump_to_last();
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(item) = self.selected_item() {
                    let label = item.label.clone();
                    EventResult::Action(WidgetAction::Selected(label))
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
            KeyCode::Char(ch) if ch.is_ascii_alphabetic() => {
                self.jump_to_letter(ch);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn sample_items() -> Vec<MenuItem> {
        vec![
            MenuItem::new("Open", "Ctrl-O"),
            MenuItem::separator(),
            MenuItem::new("Save", "Ctrl-S"),
            {
                let mut item = MenuItem::new("Disabled", "");
                item.enabled = false;
                item
            },
            MenuItem::new("Quit", "Ctrl-Q"),
        ]
    }

    #[test]
    fn selects_first_selectable() {
        let menu = Menu::new(sample_items());
        assert_eq!(menu.selected_index(), 0);
        assert_eq!(menu.selected_item().map(|i| &i.label[..]), Some("Open"));
    }

    #[test]
    fn navigation_skips_separators() {
        let mut menu = Menu::new(sample_items());
        menu.handle_key(key(KeyCode::Down));
        // Should skip separator at index 1, land on "Save" at index 2
        assert_eq!(menu.selected_index(), 2);
        assert_eq!(menu.selected_item().map(|i| &i.label[..]), Some("Save"));
    }

    #[test]
    fn navigation_skips_disabled() {
        let mut menu = Menu::new(sample_items());
        menu.selected = 2; // Save
        menu.handle_key(key(KeyCode::Down));
        // Should skip disabled at index 3, land on "Quit" at index 4
        assert_eq!(menu.selected_index(), 4);
        assert_eq!(menu.selected_item().map(|i| &i.label[..]), Some("Quit"));
    }

    #[test]
    fn up_wraps_around() {
        let mut menu = Menu::new(sample_items());
        // At "Open" (0), going up should wrap to "Quit" (4)
        menu.handle_key(key(KeyCode::Up));
        assert_eq!(menu.selected_index(), 4);
    }

    #[test]
    fn down_wraps_around() {
        let mut menu = Menu::new(sample_items());
        menu.selected = 4; // Quit
        menu.handle_key(key(KeyCode::Down));
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn first_letter_jump() {
        let mut menu = Menu::new(sample_items());
        menu.handle_key(key(KeyCode::Char('q')));
        assert_eq!(menu.selected_item().map(|i| &i.label[..]), Some("Quit"));
    }

    #[test]
    fn first_letter_skips_disabled() {
        let mut menu = Menu::new(sample_items());
        menu.handle_key(key(KeyCode::Char('d')));
        // "Disabled" is not selectable, should stay on "Open"
        assert_eq!(menu.selected_index(), 0);
    }

    #[test]
    fn enter_produces_selected() {
        let mut menu = Menu::new(sample_items());
        let result = menu.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Selected(s)) if s == "Open"
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut menu = Menu::new(sample_items());
        let result = menu.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn home_end() {
        let mut menu = Menu::new(sample_items());
        menu.handle_key(key(KeyCode::End));
        assert_eq!(menu.selected_item().map(|i| &i.label[..]), Some("Quit"));
        menu.handle_key(key(KeyCode::Home));
        assert_eq!(menu.selected_item().map(|i| &i.label[..]), Some("Open"));
    }

    #[test]
    fn empty_menu() {
        let mut menu = Menu::new(vec![]);
        assert!(menu.selected_item().is_none());
        let result = menu.handle_key(key(KeyCode::Down));
        assert!(matches!(result, EventResult::Ignored));
        let result = menu.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn separator_only_menu() {
        let menu = Menu::new(vec![MenuItem::separator()]);
        assert!(menu.selected_item().is_none());
    }

    #[test]
    fn width_and_height() {
        let menu = Menu::new(sample_items());
        assert!(menu.width() > 2);
        assert_eq!(menu.height(), 7); // 5 items + 2 border
    }

    #[test]
    fn render_no_panic() {
        let menu = Menu::new(sample_items());
        let mut screen = txv::screen::Screen::with_color_mode(30, 10, txv::cell::ColorMode::Rgb);
        let mut s = screen.full_surface();
        menu.render(&mut s, true);
    }
}
