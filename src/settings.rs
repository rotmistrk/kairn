//! Application and editor settings (3-tier: global → editor_defaults → instance).

use txv_core::prelude::*;

/// Per-editor-instance settings, cloned from AppSettings::editor_defaults on creation.
#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub(crate) wrap: bool,
    pub(crate) list: bool,
    pub(crate) tabstop: u16,
    pub(crate) number: bool,
    pub(crate) autosave: bool,
    pub(crate) autosave_delay: u16,
    pub(crate) cursor_insert: CursorStyle,
    pub(crate) cursor_normal: CursorStyle,
    pub(crate) cursor_command: CursorStyle,
}

impl EditorSettings {
    pub fn set_autosave(&mut self, v: bool) {
        self.autosave = v;
    }
}

/// Cursor style: software (reverse block) or hardware (bar/block/underline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Software,
    Bar,
    Block,
    Underline,
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
            cursor_insert: CursorStyle::Bar,
            cursor_normal: CursorStyle::Software,
            cursor_command: CursorStyle::Software,
        }
    }
}

/// Key bindings for the git changes panel.
#[derive(Debug, Clone)]
pub struct GitKeys {
    pub(crate) stage: KeyEvent,
    pub(crate) unstage: KeyEvent,
    pub(crate) untrack: KeyEvent,
    pub(crate) commit: KeyEvent,
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
    pub(crate) clock_interval: u16,
    pub(crate) scrollback_lines: u16,
    pub(crate) max_tabs: u16,
    pub(crate) theme_mode: String,
    pub(crate) theme_syntax_dark: String,
    pub(crate) theme_syntax_light: String,
    pub(crate) theme_glyphs: String,
    pub(crate) editor_defaults: EditorSettings,
    pub(crate) build_command: Option<String>,
    pub(crate) run_command: Option<String>,
    pub(crate) test_command: Option<String>,
    pub(crate) lsp_timeout: u64,
    pub(crate) git_keys: GitKeys,
    pub(crate) status_keys: StatusKeys,
    /// Seconds before a terminal tab is considered idle.
    pub(crate) terminal_idle_timeout: u64,
    /// Auto-close terminal tabs on exit.
    pub(crate) terminal_auto_close: bool,
    /// Width threshold to switch from tall to wide layout.
    pub(crate) layout_wide_threshold: u16,
    /// Width threshold to switch from wide to tall layout.
    pub(crate) layout_tall_threshold: u16,
}

/// Key bindings for the status bar (visible labels).
#[derive(Debug, Clone)]
pub struct StatusKeys {
    pub(crate) help: KeyEvent,
    pub(crate) tree: KeyEvent,
    pub(crate) main: KeyEvent,
    pub(crate) term: KeyEvent,
    pub(crate) zoom: KeyEvent,
    pub(crate) messages: KeyEvent,
    pub(crate) quit: KeyEvent,
    pub(crate) subpanel_focus: KeyEvent,
    pub(crate) subpanel_move: KeyEvent,
    pub(crate) subpanel_grow: KeyEvent,
    pub(crate) subpanel_shrink: KeyEvent,
}

impl Default for StatusKeys {
    fn default() -> Self {
        Self {
            help: fkey(1),
            tree: fkey(2),
            main: fkey(3),
            term: fkey(4),
            zoom: fkey(5),
            messages: fkey(6),
            quit: ctrl_key('q'),
            subpanel_focus: ctrl_key('w'),
            subpanel_move: ctrl_alt_key('w'),
            subpanel_grow: ctrl_alt_key('='),
            subpanel_shrink: ctrl_alt_key('-'),
        }
    }
}

fn fkey(n: u8) -> KeyEvent {
    KeyEvent {
        code: KeyCode::F(n),
        modifiers: KeyMod::default(),
    }
}

fn ctrl_key(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    }
}

fn ctrl_alt_key(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: true,
            alt: true,
            shift: false,
        },
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
            theme_glyphs: "nerd".to_string(),
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
    pub fn git_keys(&self) -> &GitKeys {
        &self.git_keys
    }
    pub fn theme_mode(&self) -> &str {
        &self.theme_mode
    }
    pub fn theme_glyphs(&self) -> &str {
        &self.theme_glyphs
    }
    pub fn layout_wide_threshold(&self) -> u16 {
        self.layout_wide_threshold
    }
    pub fn editor_defaults(&self) -> &EditorSettings {
        &self.editor_defaults
    }
    pub fn editor_defaults_mut(&mut self) -> &mut EditorSettings {
        &mut self.editor_defaults
    }
    pub fn clock_interval(&self) -> u16 {
        self.clock_interval
    }
    pub fn status_keys(&self) -> &StatusKeys {
        &self.status_keys
    }
    pub fn set_max_tabs(&mut self, v: u16) {
        self.max_tabs = v;
    }
    /// Returns the syntax theme name for the current mode.
    pub fn syntax_theme_for_mode(&self, is_light: bool) -> &str {
        if is_light {
            &self.theme_syntax_light
        } else {
            &self.theme_syntax_dark
        }
    }
}
