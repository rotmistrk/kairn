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
    pub editor_defaults: EditorSettings,
    pub build_command: Option<String>,
    pub run_command: Option<String>,
    pub test_command: Option<String>,
    pub git_keys: GitKeys,
    pub status_keys: StatusKeys,
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
            editor_defaults: EditorSettings::default(),
            build_command: None,
            run_command: None,
            test_command: None,
            git_keys: GitKeys::default(),
            status_keys: StatusKeys::default(),
        }
    }
}
