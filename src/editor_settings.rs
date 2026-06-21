//! Per-editor-instance settings.

pub use txv_edit::settings::CursorStyle;

/// Per-editor-instance settings, cloned from AppSettings::editor_defaults on creation.
#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub(crate) wrap: bool,
    pub(crate) list: bool,
    pub(crate) tabstop: u16,
    pub(crate) number: bool,
    pub(crate) autosave: bool,
    pub(crate) autosave_delay: u16,
    pub(crate) autocomplete: bool,
    pub(crate) rainbow: bool,
    pub(crate) guides: bool,
    pub(crate) gutter_signs: bool,
    pub(crate) scrolloff: usize,
    pub(crate) cursor_insert: CursorStyle,
    pub(crate) cursor_normal: CursorStyle,
    pub(crate) cursor_command: CursorStyle,
}

impl EditorSettings {
    pub fn set_autosave(&mut self, v: bool) {
        self.autosave = v;
    }
    pub fn set_wrap(&mut self, v: bool) {
        self.wrap = v;
    }
    pub fn set_list(&mut self, v: bool) {
        self.list = v;
    }
    pub fn set_number(&mut self, v: bool) {
        self.number = v;
    }
    pub fn set_rainbow(&mut self, v: bool) {
        self.rainbow = v;
    }
    pub fn set_guides(&mut self, v: bool) {
        self.guides = v;
    }
    pub fn set_gutter_signs(&mut self, v: bool) {
        self.gutter_signs = v;
    }
    pub fn set_cursor_normal(&mut self, v: CursorStyle) {
        self.cursor_normal = v;
    }
    pub fn set_cursor_insert(&mut self, v: CursorStyle) {
        self.cursor_insert = v;
    }
    pub fn set_cursor_command(&mut self, v: CursorStyle) {
        self.cursor_command = v;
    }
    pub fn autosave(&self) -> bool {
        self.autosave
    }
    pub fn autocomplete(&self) -> bool {
        self.autocomplete
    }
    pub fn set_autocomplete(&mut self, v: bool) {
        self.autocomplete = v;
    }
    pub fn gutter_signs(&self) -> bool {
        self.gutter_signs
    }
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
            autocomplete: false,
            rainbow: true,
            guides: true,
            gutter_signs: true,
            scrolloff: 3,
            cursor_insert: CursorStyle::Bar,
            cursor_normal: CursorStyle::Software,
            cursor_command: CursorStyle::Software,
        }
    }
}
