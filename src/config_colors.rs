//! Color configuration loading from Tcl variables.

use std::sync::Arc;

use rusticle::interpreter::Interpreter;
use txv_core::cell::{Attrs, Color, Style};
use txv_core::palette::{Palette, StyleId};

use crate::app_palette::AppPalette;
use crate::custom_palette::CustomPalette;

/// Accessor function that extracts a mutable Style ref from AppPalette.
type StyleAccessor = fn(&mut AppPalette) -> &mut Style;

/// Table of (config_key, accessor) pairs for foreground color overrides.
const FG_ENTRIES: &[(&str, StyleAccessor)] = &[
    // Git
    ("color.git.added", |p| p.git_mut().added_mut()),
    ("color.git.modified", |p| p.git_mut().modified_mut()),
    ("color.git.untracked", |p| p.git_mut().untracked_mut()),
    ("color.git.ignored", |p| p.git_mut().ignored_mut()),
    ("color.git.conflict", |p| p.git_mut().conflict_mut()),
    // Diff
    ("color.diff.added", |p| p.diff_mut().added_mut()),
    ("color.diff.deleted", |p| p.diff_mut().deleted_mut()),
    ("color.diff.fold", |p| p.diff_mut().fold_mut()),
    // Editor
    ("color.editor.gutter", |p| p.editor_mut().gutter_mut()),
    ("color.editor.list_chars", |p| p.editor_mut().list_chars_mut()),
    // Diagnostics
    ("color.diag.error", |p| p.diag_mut().error_mut()),
    ("color.diag.warning", |p| p.diag_mut().warning_mut()),
    ("color.diag.info", |p| p.diag_mut().info_mut()),
    ("color.diag.hint", |p| p.diag_mut().hint_mut()),
    // Tree
    ("color.tree.directory", |p| p.tree_mut().directory_mut()),
    // Todo
    ("color.todo.normal", |p| p.todo_mut().normal_mut()),
    ("color.todo.done", |p| p.todo_mut().done_mut()),
    ("color.todo.important", |p| p.todo_mut().important_mut()),
    // Messages
    ("color.msg.error", |p| p.msg_mut().error_mut()),
    ("color.msg.warning", |p| p.msg_mut().warning_mut()),
    ("color.msg.info", |p| p.msg_mut().info_mut()),
    ("color.msg.debug", |p| p.msg_mut().debug_mut()),
];

/// Apply color overrides from config to the palette.
pub fn apply_color_config(interp: &Interpreter, palette: &mut AppPalette) {
    for &(key, accessor) in FG_ENTRIES {
        apply_fg(interp, key, accessor(palette));
    }
}

/// Chrome style override table.
const CHROME_ENTRIES: &[(&str, StyleId)] = &[
    ("color.chrome.status_bar", StyleId::StatusBar),
    ("color.chrome.status_bar_modal", StyleId::StatusBarModal),
    ("color.chrome.bar", StyleId::ChromeBar),
    ("color.chrome.tab_focused", StyleId::TabFocused),
    ("color.chrome.tab_active", StyleId::TabActive),
    ("color.chrome.scrollbar_track", StyleId::ScrollbarTrack),
    ("color.chrome.scrollbar_thumb", StyleId::ScrollbarThumb),
    ("color.chrome.status_question", StyleId::StatusQuestion),
    ("color.chrome.status_highlight", StyleId::StatusHighlight),
    ("color.popup.background", StyleId::PopupBackground),
    ("color.popup.border", StyleId::PopupBorder),
    ("color.popup.selected", StyleId::PopupSelected),
    ("color.interactive.cursor_focused", StyleId::CursorFocused),
    ("color.interactive.input_cursor", StyleId::InputCursor),
    ("color.interactive.search_match", StyleId::SearchMatch),
];

/// Apply chrome/framework color overrides. Returns a CustomPalette if any overrides set.
pub fn apply_chrome_config(interp: &Interpreter, base: Arc<dyn Palette>) -> Arc<dyn Palette> {
    let mut custom = CustomPalette::new(base.clone());
    let mut has_overrides = false;

    for &(var, id) in CHROME_ENTRIES {
        if let Some(style) = parse_style(interp, var) {
            custom.set_override(id, style);
            has_overrides = true;
        }
    }

    if has_overrides {
        Arc::new(custom)
    } else {
        base
    }
}

/// Parse a style from "fg [bg [attrs]]" format.
/// Examples: "7", "7 236", "15 18 bold"
fn parse_style(interp: &Interpreter, var: &str) -> Option<Style> {
    let val = interp.get_var(var)?;
    let s = val.as_str();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let fg = parse_color(parts[0])?;
    let bg = parts.get(1).and_then(|p| parse_color(p)).unwrap_or(Color::Reset);
    let attrs = parse_attrs(&parts[2..]);
    Some(Style::new(fg, bg).with_attrs(attrs))
}

/// Parse attribute flags from string parts.
fn parse_attrs(parts: &[&str]) -> Attrs {
    let mut attrs = Attrs::default();
    for part in parts {
        match *part {
            "bold" => attrs.set_bold(true),
            "italic" => attrs.set_italic(true),
            "underline" => attrs.set_underline(true),
            "dim" => attrs.set_dim(true),
            _ => {}
        }
    }
    attrs
}

/// Parse a color: number (ansi 0-15), "p:N" (palette 0-255), "rgb:RRGGBB"
fn parse_color(s: &str) -> Option<Color> {
    if let Some(rest) = s.strip_prefix("p:") {
        return rest.parse::<u8>().ok().map(Color::Palette);
    }
    if let Some(rest) = s.strip_prefix("rgb:") {
        if rest.len() == 6 {
            let r = u8::from_str_radix(&rest[0..2], 16).ok()?;
            let g = u8::from_str_radix(&rest[2..4], 16).ok()?;
            let b = u8::from_str_radix(&rest[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    s.parse::<u8>().ok().map(Color::Ansi)
}

fn apply_fg(interp: &Interpreter, var: &str, style: &mut Style) {
    if let Some(val) = interp.get_var(var) {
        if let Ok(n) = val.as_int() {
            style.set_fg(Color::Ansi(n as u8));
        }
    }
}
