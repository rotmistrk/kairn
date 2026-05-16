//! Application and editor settings (3-tier: global → editor_defaults → instance).

use txv_core::prelude::*;

/// Per-editor-instance settings, cloned from AppSettings::editor_defaults on creation.
#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub wrap: bool,
    pub list: bool,
    pub tabstop: u16,
    pub number: bool,
    pub autosave: bool,
    pub autosave_delay: u16,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            wrap: true,
            list: false,
            tabstop: 4,
            number: true,
            autosave: true,
            autosave_delay: 5,
        }
    }
}

/// Key bindings for the git changes panel.
#[derive(Debug, Clone)]
pub struct GitKeys {
    pub stage: KeyEvent,
    pub unstage: KeyEvent,
    pub untrack: KeyEvent,
    pub commit: KeyEvent,
}

impl Default for GitKeys {
    fn default() -> Self {
        Self {
            stage: KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyMod::default(),
            },
            unstage: KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyMod::default(),
            },
            untrack: KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyMod::default(),
            },
            commit: KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyMod::default(),
            },
        }
    }
}

/// Global application settings.
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub clock_interval: u16,
    pub scrollback_lines: u16,
    pub max_tabs: u16,
    pub theme_mode: String,
    pub theme_syntax_dark: String,
    pub theme_syntax_light: String,
    pub theme_glyphs: String,
    pub editor_defaults: EditorSettings,
    pub build_command: Option<String>,
    pub run_command: Option<String>,
    pub test_command: Option<String>,
    pub lsp_timeout: u64,
    pub git_keys: GitKeys,
    pub status_keys: StatusKeys,
    /// Seconds before a terminal tab is considered idle.
    pub terminal_idle_timeout: u64,
    /// Auto-close terminal tabs on exit.
    pub terminal_auto_close: bool,
    /// Width threshold to switch from tall to wide layout.
    pub layout_wide_threshold: u16,
    /// Width threshold to switch from wide to tall layout.
    pub layout_tall_threshold: u16,
}

/// Key bindings for the status bar (visible labels).
#[derive(Debug, Clone)]
pub struct StatusKeys {
    pub help: KeyEvent,
    pub tree: KeyEvent,
    pub main: KeyEvent,
    pub term: KeyEvent,
    pub zoom: KeyEvent,
    pub messages: KeyEvent,
    pub quit: KeyEvent,
}

impl Default for StatusKeys {
    fn default() -> Self {
        Self {
            help: KeyEvent {
                code: KeyCode::F(1),
                modifiers: KeyMod::default(),
            },
            tree: KeyEvent {
                code: KeyCode::F(2),
                modifiers: KeyMod::default(),
            },
            main: KeyEvent {
                code: KeyCode::F(3),
                modifiers: KeyMod::default(),
            },
            term: KeyEvent {
                code: KeyCode::F(4),
                modifiers: KeyMod::default(),
            },
            zoom: KeyEvent {
                code: KeyCode::F(5),
                modifiers: KeyMod::default(),
            },
            messages: KeyEvent {
                code: KeyCode::F(6),
                modifiers: KeyMod::default(),
            },
            quit: KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyMod {
                    ctrl: true,
                    alt: false,
                    shift: false,
                },
            },
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            clock_interval: 60,
            scrollback_lines: 2000,
            max_tabs: 10,
            theme_mode: "auto".to_string(),
            theme_syntax_dark: "base16-eighties.dark".to_string(),
            theme_syntax_light: "base16-ocean.light".to_string(),
            theme_glyphs: "auto".to_string(),
            editor_defaults: EditorSettings::default(),
            build_command: None,
            run_command: None,
            test_command: None,
            lsp_timeout: 10,
            git_keys: GitKeys::default(),
            status_keys: StatusKeys::default(),
            terminal_idle_timeout: 3,
            terminal_auto_close: true,
            layout_wide_threshold: 300,
            layout_tall_threshold: 200,
        }
    }
}

impl AppSettings {
    /// Returns the syntax theme name for the current mode.
    pub fn syntax_theme_for_mode(&self, is_light: bool) -> &str {
        if is_light {
            &self.theme_syntax_light
        } else {
            &self.theme_syntax_dark
        }
    }
}
