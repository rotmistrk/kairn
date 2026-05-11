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
        }
    }
}
