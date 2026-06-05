//! Per-editor-instance settings.

/// Cursor style: software (reverse block) or hardware (bar/block/underline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Software,
    Bar,
    Block,
    Underline,
}

/// Per-editor-instance settings, cloned from AppSettings::editor_defaults on creation.
#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub(crate) wrap: bool,
    pub(crate) list: bool,
    pub(crate) tabstop: u16,
    pub(crate) number: bool,
    pub(crate) autosave: bool,
    pub(crate) autosave_delay: u16,
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
