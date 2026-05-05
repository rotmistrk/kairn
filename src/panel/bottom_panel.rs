//! Bottom panel: tabbed container for terminal, kiro, errors, tests, output.

use crossterm::event::KeyEvent;
use txv::surface::Surface;
use txv_widgets::{EventResult, TabBar, TabEntry, Widget};

use super::terminal_panel::TerminalPanel;
use crate::runner::tests::{TestResult, TestStatus, TestSuite};
use crate::runner::{BuildError, Severity};

/// A tab in the bottom panel.
pub enum BottomTab {
    /// Shell terminal.
    Terminal(TerminalPanel),
    /// Kiro AI terminal.
    Kiro(TerminalPanel),
    /// Build errors list.
    Errors(ErrorsTab),
    /// Test results.
    Tests(TestsTab),
    /// Output text.
    Output(Vec<String>),
}

/// Errors tab: list of parsed build errors.
pub struct ErrorsTab {
    pub errors: Vec<BuildError>,
    pub selected: usize,
}

impl ErrorsTab {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            selected: 0,
        }
    }

    /// Navigate to next error, wrapping around.
    pub fn next(&mut self) {
        if !self.errors.is_empty() {
            self.selected = (self.selected + 1) % self.errors.len();
        }
    }

    /// Navigate to previous error, wrapping around.
    pub fn prev(&mut self) {
        if !self.errors.is_empty() {
            let len = self.errors.len();
            self.selected = (self.selected + len - 1) % len;
        }
    }

    /// Get the currently selected error.
    pub fn current(&self) -> Option<&BuildError> {
        self.errors.get(self.selected)
    }
}

/// Tests tab: tree of test suites with pass/fail.
pub struct TestsTab {
    pub suites: Vec<TestSuite>,
    pub selected: usize,
}

impl TestsTab {
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
            selected: 0,
        }
    }

    /// Get all results flattened.
    pub fn all_results(&self) -> Vec<&TestResult> {
        self.suites.iter().flat_map(|s| &s.results).collect()
    }

    /// Navigate to next result.
    pub fn next(&mut self) {
        let total = self.all_results().len();
        if total > 0 {
            self.selected = (self.selected + 1) % total;
        }
    }

    /// Navigate to previous result.
    pub fn prev(&mut self) {
        let total = self.all_results().len();
        if total > 0 {
            self.selected = (self.selected + total - 1) % total;
        }
    }
}

impl BottomTab {
    fn title(&self) -> &str {
        match self {
            Self::Terminal(t) => t.title(),
            Self::Kiro(t) => t.title(),
            Self::Errors(_) => "errors",
            Self::Tests(_) => "tests",
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

    /// Get reference to a tab by index.
    pub fn tab_at(&self, idx: usize) -> Option<&BottomTab> {
        self.tabs.get(idx)
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
                BottomTab::Errors(_) | BottomTab::Tests(_) | BottomTab::Output(_) => {}
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
                    BottomTab::Errors(et) => {
                        render_errors(&mut content, et);
                    }
                    BottomTab::Tests(tt) => {
                        render_tests(&mut content, tt);
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
                BottomTab::Errors(_) | BottomTab::Tests(_) | BottomTab::Output(_) => {
                    EventResult::Ignored
                }
            }
        } else {
            EventResult::Ignored
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}

/// Render the errors tab content.
fn render_errors(surface: &mut Surface<'_>, tab: &ErrorsTab) {
    use txv::cell::{Color, Style};
    let h = surface.height() as usize;
    for (i, err) in tab.errors.iter().enumerate() {
        if i >= h {
            break;
        }
        let prefix = match err.severity {
            Severity::Error => "E",
            Severity::Warning => "W",
            Severity::Info => "I",
        };
        let line = format!(
            " {prefix} {}:{}:{} {}",
            err.file, err.line, err.col, err.message
        );
        let style = if i == tab.selected {
            Style {
                fg: Color::Rgb(235, 219, 178),
                bg: Color::Palette(237),
                ..Style::default()
            }
        } else {
            Style::default()
        };
        surface.print(0, i as u16, &line, style);
    }
}

/// Render the tests tab content.
fn render_tests(surface: &mut Surface<'_>, tab: &TestsTab) {
    use txv::cell::{Color, Style};
    let h = surface.height() as usize;
    let mut row = 0;
    for suite in &tab.suites {
        if row >= h {
            break;
        }
        let header = format!(
            " {} ({}/{} passed)",
            suite.name,
            suite.passed(),
            suite.total()
        );
        surface.print(0, row as u16, &header, Style::default());
        row += 1;
        for result in &suite.results {
            if row >= h {
                break;
            }
            let icon = match result.status {
                TestStatus::Pass => "✓",
                TestStatus::Fail => "✗",
                TestStatus::Skip => "○",
            };
            let fg = match result.status {
                TestStatus::Pass => Color::Rgb(142, 192, 124),
                TestStatus::Fail => Color::Rgb(251, 73, 52),
                TestStatus::Skip => Color::Rgb(168, 153, 132),
            };
            let style = Style {
                fg,
                ..Style::default()
            };
            let line = format!("   {icon} {}", result.name);
            surface.print(0, row as u16, &line, style);
            row += 1;
        }
    }
}
