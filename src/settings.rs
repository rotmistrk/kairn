//! Application and editor settings (3-tier: global → editor_defaults → instance).

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

/// Global application settings.
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub clock_interval: u16,
    pub editor_defaults: EditorSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            clock_interval: 60,
            editor_defaults: EditorSettings::default(),
        }
    }
}
