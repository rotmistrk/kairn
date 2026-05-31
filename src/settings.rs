//! Global application settings.

pub use crate::editor_settings::{CursorStyle, EditorSettings};
pub use crate::git_keys::GitKeys;
pub use crate::kiro_settings::KiroLaunchSettings;
pub use crate::status_keys::StatusKeys;

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
    pub(crate) terminal_idle_timeout: u64,
    pub(crate) terminal_auto_close: bool,
    pub(crate) layout_wide_threshold: u16,
    pub(crate) layout_tall_threshold: u16,
    pub(crate) kiro: KiroLaunchSettings,
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
            kiro: KiroLaunchSettings::default(),
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
    pub fn syntax_theme_for_mode(&self, is_light: bool) -> &str {
        if is_light {
            &self.theme_syntax_light
        } else {
            &self.theme_syntax_dark
        }
    }
    pub fn kiro(&self) -> &KiroLaunchSettings {
        &self.kiro
    }
}
