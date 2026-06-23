//! UI chrome state: waker, tty, window title, pty output, tab titles, keys, theme.

use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;

use txv_core::run::Waker;

use crate::theme_state::ThemeState;

/// UI chrome state.
pub(crate) struct UiChrome {
    waker: Option<Waker>,
    tty_file: Option<std::fs::File>,
    last_window_title: String,
    pty_last_output: HashMap<usize, Instant>,
    tab_titles_dirty: bool,
    show_messages_on_start: bool,
    key_bindings: Vec<txv_core::key_help::KeyHelpEntry>,
    theme_state: Option<RefCell<ThemeState>>,
}

impl UiChrome {
    pub(crate) fn new() -> Self {
        Self {
            waker: None,
            tty_file: None,
            last_window_title: String::new(),
            pty_last_output: HashMap::new(),
            tab_titles_dirty: false,
            show_messages_on_start: false,
            key_bindings: Vec::new(),
            theme_state: None,
        }
    }

    pub(crate) fn waker(&self) -> &Option<Waker> {
        &self.waker
    }

    pub(crate) fn set_waker(&mut self, w: Waker) {
        self.waker = Some(w);
    }

    pub(crate) fn tty_file(&self) -> &Option<std::fs::File> {
        &self.tty_file
    }

    pub(crate) fn tty_file_mut(&mut self) -> &mut Option<std::fs::File> {
        &mut self.tty_file
    }

    pub(crate) fn set_tty_file(&mut self, f: Option<std::fs::File>) {
        self.tty_file = f;
    }

    pub(crate) fn last_window_title(&self) -> &str {
        &self.last_window_title
    }

    pub(crate) fn set_last_window_title(&mut self, t: String) {
        self.last_window_title = t;
    }

    pub(crate) fn pty_last_output(&self) -> &HashMap<usize, Instant> {
        &self.pty_last_output
    }

    pub(crate) fn record_pty_output(&mut self, index: usize, when: Instant) {
        self.pty_last_output.insert(index, when);
    }

    pub(crate) fn tab_titles_dirty(&self) -> bool {
        self.tab_titles_dirty
    }

    pub(crate) fn set_tab_titles_dirty(&mut self, v: bool) {
        self.tab_titles_dirty = v;
    }

    pub(crate) fn show_messages_on_start(&self) -> bool {
        self.show_messages_on_start
    }

    pub(crate) fn set_show_messages_on_start(&mut self, v: bool) {
        self.show_messages_on_start = v;
    }

    pub(crate) fn key_bindings(&self) -> &[txv_core::key_help::KeyHelpEntry] {
        &self.key_bindings
    }

    pub(crate) fn set_key_bindings(&mut self, b: Vec<txv_core::key_help::KeyHelpEntry>) {
        self.key_bindings = b;
    }

    pub(crate) fn theme_state(&self) -> &Option<RefCell<ThemeState>> {
        &self.theme_state
    }

    pub(crate) fn set_theme_state(&mut self, ts: ThemeState) {
        self.theme_state = Some(RefCell::new(ts));
    }
}
