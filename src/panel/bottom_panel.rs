//! Bottom panel: tabbed container for terminal, kiro, errors, etc.

use crossterm::event::KeyEvent;
use txv::surface::Surface;
use txv_widgets::{EventResult, TabBar, TabEntry, Widget};

use super::terminal_panel::TerminalPanel;

/// A tab in the bottom panel.
pub enum BottomTab {
    /// Shell terminal.
    Terminal(TerminalPanel),
    /// Kiro AI terminal.
    Kiro(TerminalPanel),
    /// Output text.
    Output(Vec<String>),
}

impl BottomTab {
    fn title(&self) -> &str {
        match self {
            Self::Terminal(t) => t.title(),
            Self::Kiro(t) => t.title(),
            Self::Output(_) => "output",
        }
    }
}

/// Tabbed bottom panel.
pub struct BottomPanel {
    pub tab_bar: TabBar,
    tabs: Vec<BottomTab>,
    pub visible: bool,
}

impl BottomPanel {
    /// Create a new empty bottom panel.
    pub fn new() -> Self {
        Self {
            tab_bar: TabBar::new(),
            tabs: Vec::new(),
            visible: true,
        }
    }

    /// Add a terminal tab.
    pub fn add_terminal(&mut self, tab: BottomTab) {
        let title = tab.title().to_string();
        self.tabs.push(tab);
        self.tab_bar.add(TabEntry {
            title,
            modified: false,
        });
        self.tab_bar.set_active(self.tabs.len() - 1);
    }

    /// Get the active tab index.
    pub fn active_index(&self) -> usize {
        self.tab_bar.active()
    }

    /// Get mutable reference to active tab.
    pub fn active_tab_mut(&mut self) -> Option<&mut BottomTab> {
        let idx = self.tab_bar.active();
        self.tabs.get_mut(idx)
    }

    /// Get reference to active tab.
    pub fn active_tab(&self) -> Option<&BottomTab> {
        let idx = self.tab_bar.active();
        self.tabs.get(idx)
    }

    /// Number of tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Switch to next tab.
    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let next = (self.tab_bar.active() + 1) % self.tabs.len();
        self.tab_bar.set_active(next);
    }

    /// Switch to previous tab.
    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let len = self.tabs.len();
        let prev = (self.tab_bar.active() + len - 1) % len;
        self.tab_bar.set_active(prev);
    }

    /// Close the active tab.
    pub fn close_active_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        let idx = self.tab_bar.active();
        self.tabs.remove(idx);
        self.tab_bar.remove(idx);
        if !self.tabs.is_empty() {
            let new_active = idx.min(self.tabs.len() - 1);
            self.tab_bar.set_active(new_active);
        }
    }

    /// Process PTY output for a specific tab by poller index.
    pub fn process_poll_data(&mut self, tab_idx: usize, data: &[u8]) {
        if let Some(tab) = self.tabs.get_mut(tab_idx) {
            match tab {
                BottomTab::Terminal(t) | BottomTab::Kiro(t) => {
                    t.process_output(data);
                }
                BottomTab::Output(_) => {}
            }
        }
    }

    /// Toggle visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl Widget for BottomPanel {
    fn render(&self, surface: &mut Surface<'_>, focused: bool) {
        let h = surface.height();
        if h < 2 {
            return;
        }

        // Tab bar in first row.
        {
            let mut tab_surface = surface.sub(0, 0, surface.width(), 1);
            self.tab_bar.render(&mut tab_surface, false);
        }

        // Active tab content in remaining rows.
        if h > 1 {
            let mut content = surface.sub(0, 1, surface.width(), h - 1);
            if let Some(tab) = self.active_tab() {
                match tab {
                    BottomTab::Terminal(t) | BottomTab::Kiro(t) => {
                        t.render(&mut content, focused);
                    }
                    BottomTab::Output(lines) => {
                        for (i, line) in lines.iter().enumerate() {
                            let row = i as u16;
                            if row >= content.height() {
                                break;
                            }
                            content.print(0, row, line, txv::cell::Style::default());
                        }
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if let Some(tab) = self.active_tab_mut() {
            match tab {
                BottomTab::Terminal(t) | BottomTab::Kiro(t) => t.handle_key(key),
                BottomTab::Output(_) => EventResult::Ignored,
            }
        } else {
            EventResult::Ignored
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}
