//! Horizontal tab strip widget.

use crossterm::event::KeyEvent;
use txv::cell::Style;
use txv::surface::Surface;
use txv::text::display_width;

use crate::widget::{EventResult, Widget};

/// A single tab entry.
pub struct TabEntry {
    /// Tab title.
    pub title: String,
    /// Whether the tab has unsaved changes.
    pub modified: bool,
}

/// Horizontal tab bar showing a strip of tabs.
pub struct TabBar {
    tabs: Vec<TabEntry>,
    active: usize,
    /// Style for the active tab.
    pub active_style: Style,
    /// Style for inactive tabs.
    pub inactive_style: Style,
}

impl TabBar {
    /// Create an empty tab bar.
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            active_style: Style {
                attrs: txv::cell::Attrs {
                    bold: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            inactive_style: Style::default(),
        }
    }

    /// Add a tab.
    pub fn add(&mut self, entry: TabEntry) {
        self.tabs.push(entry);
    }

    /// Remove a tab by index. Adjusts active index if needed.
    pub fn remove(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active = 0;
        } else if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
    }

    /// Get the active tab index.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Set the active tab index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
        }
    }

    /// Get the active tab's title, or empty if no tabs.
    pub fn active_title(&self) -> &str {
        self.tabs
            .get(self.active)
            .map(|t| t.title.as_str())
            .unwrap_or("")
    }

    /// Number of tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Whether there are no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Get a tab entry by index.
    pub fn get(&self, index: usize) -> Option<&TabEntry> {
        self.tabs.get(index)
    }

    fn tab_label(&self, index: usize) -> String {
        let tab = &self.tabs[index];
        let marker = if index == self.active { "▸" } else { " " };
        let modified = if tab.modified { "[+]" } else { "" };
        format!("{marker}{}{modified} ", tab.title)
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TabBar {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let w = surface.width();
        surface.hline(0, 0, w, ' ', self.inactive_style);

        let mut col: u16 = 0;
        for i in 0..self.tabs.len() {
            let label = self.tab_label(i);
            let lw = display_width(&label) as u16;
            if col + lw > w {
                break;
            }
            let style = if i == self.active {
                self.active_style
            } else {
                self.inactive_style
            };
            surface.print(col, 0, &label, style);
            col += lw;
        }
    }

    fn handle_key(&mut self, _key: KeyEvent) -> EventResult {
        EventResult::Ignored
    }

    fn focusable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn render_bar(bar: &TabBar, width: u16) -> String {
        let mut screen = Screen::with_color_mode(width, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            bar.render(&mut s, false);
        }
        screen.to_text().trim_end_matches('\n').to_string()
    }

    #[test]
    fn empty_bar() {
        let bar = TabBar::new();
        assert!(bar.is_empty());
        assert_eq!(bar.len(), 0);
        assert_eq!(bar.active_title(), "");
    }

    #[test]
    fn add_and_active() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "Tab1".into(),
            modified: false,
        });
        bar.add(TabEntry {
            title: "Tab2".into(),
            modified: false,
        });
        assert_eq!(bar.len(), 2);
        assert_eq!(bar.active(), 0);
        assert_eq!(bar.active_title(), "Tab1");
    }

    #[test]
    fn set_active() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "A".into(),
            modified: false,
        });
        bar.add(TabEntry {
            title: "B".into(),
            modified: false,
        });
        bar.set_active(1);
        assert_eq!(bar.active(), 1);
        assert_eq!(bar.active_title(), "B");
    }

    #[test]
    fn set_active_out_of_bounds() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "A".into(),
            modified: false,
        });
        bar.set_active(5);
        assert_eq!(bar.active(), 0); // unchanged
    }

    #[test]
    fn remove_adjusts_active() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "A".into(),
            modified: false,
        });
        bar.add(TabEntry {
            title: "B".into(),
            modified: false,
        });
        bar.set_active(1);
        bar.remove(1);
        assert_eq!(bar.active(), 0);
        assert_eq!(bar.len(), 1);
    }

    #[test]
    fn remove_last_tab() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "A".into(),
            modified: false,
        });
        bar.remove(0);
        assert!(bar.is_empty());
        assert_eq!(bar.active(), 0);
    }

    #[test]
    fn render_shows_active_marker() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "T1".into(),
            modified: false,
        });
        bar.add(TabEntry {
            title: "T2".into(),
            modified: false,
        });
        let text = render_bar(&bar, 30);
        assert!(text.contains("▸T1"));
        assert!(text.contains(" T2"));
    }

    #[test]
    fn render_modified_marker() {
        let mut bar = TabBar::new();
        bar.add(TabEntry {
            title: "F".into(),
            modified: true,
        });
        let text = render_bar(&bar, 20);
        assert!(text.contains("[+]"));
    }

    #[test]
    fn not_focusable() {
        let bar = TabBar::new();
        assert!(!bar.focusable());
    }

    #[test]
    fn handle_key_ignored() {
        use crossterm::event::{KeyCode, KeyModifiers};
        let mut bar = TabBar::new();
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(matches!(bar.handle_key(key), EventResult::Ignored));
    }
}
