//! Color configuration loading from Tcl variables.

use rusticle::interpreter::Interpreter;
use txv_core::cell::{Attrs, Color, Style};
use txv_core::palette::style_id::StyleId;

use crate::app_palette::AppPalette;
use crate::custom_palette::CustomPalette;

/// Apply color overrides from config to the palette.
pub fn apply_color_config(interp: &Interpreter, palette: &mut AppPalette) {
    // Git colors
    apply_fg(interp, "color.git.added", palette.git_mut().added_mut());
    apply_fg(interp, "color.git.modified", palette.git_mut().modified_mut());
    apply_fg(interp, "color.git.untracked", palette.git_mut().untracked_mut());
    apply_fg(interp, "color.git.ignored", palette.git_mut().ignored_mut());
    apply_fg(interp, "color.git.conflict", palette.git_mut().conflict_mut());
    // Diff
    apply_fg(interp, "color.diff.added", palette.diff_mut().added_mut());
    apply_fg(interp, "color.diff.deleted", palette.diff_mut().deleted_mut());
    apply_fg(interp, "color.diff.fold", palette.diff_mut().fold_mut());
    // Editor
    apply_fg(interp, "color.editor.gutter", palette.editor_mut().gutter_mut());
    apply_fg(interp, "color.editor.list_chars", palette.editor_mut().list_chars_mut());
    // Diagnostics
    apply_fg(interp, "color.diag.error", palette.diag_mut().error_mut());
    apply_fg(interp, "color.diag.warning", palette.diag_mut().warning_mut());
    apply_fg(interp, "color.diag.info", palette.diag_mut().info_mut());
    apply_fg(interp, "color.diag.hint", palette.diag_mut().hint_mut());
    // Tree
    apply_fg(interp, "color.tree.directory", palette.tree_mut().directory_mut());
    // Todo
    apply_fg(interp, "color.todo.normal", palette.todo_mut().normal_mut());
    apply_fg(interp, "color.todo.done", palette.todo_mut().done_mut());
    apply_fg(interp, "color.todo.important", palette.todo_mut().important_mut());
    // Messages
    apply_fg(interp, "color.msg.error", palette.msg_mut().error_mut());
    apply_fg(interp, "color.msg.warning", palette.msg_mut().warning_mut());
    apply_fg(interp, "color.msg.info", palette.msg_mut().info_mut());
    apply_fg(interp, "color.msg.debug", palette.msg_mut().debug_mut());
}

/// Apply chrome/framework color overrides. Returns a CustomPalette if any overrides set.
pub fn apply_chrome_config(
    interp: &Interpreter,
    base: std::sync::Arc<dyn txv_core::palette::Palette>,
) -> std::sync::Arc<dyn txv_core::palette::Palette> {
    let mut custom = CustomPalette::new(base.clone());
    let mut has_overrides = false;

    let entries: &[(&str, StyleId)] = &[
        ("color.chrome.status_bar", StyleId::StatusBar),
        ("color.chrome.status_bar_modal", StyleId::StatusBarModal),
        ("color.chrome.bar", StyleId::ChromeBar),
        ("color.chrome.tab_focused", StyleId::TabFocused),
        ("color.chrome.tab_active", StyleId::TabActive),
        ("color.chrome.scrollbar_track", StyleId::ScrollbarTrack),
        ("color.chrome.scrollbar_thumb", StyleId::ScrollbarThumb),
        ("color.popup.background", StyleId::PopupBackground),
        ("color.popup.border", StyleId::PopupBorder),
        ("color.popup.selected", StyleId::PopupSelected),
        ("color.interactive.cursor_focused", StyleId::CursorFocused),
        ("color.interactive.input_cursor", StyleId::InputCursor),
        ("color.interactive.search_match", StyleId::SearchMatch),
    ];

    for &(var, id) in entries {
        if let Some(style) = parse_style(interp, var) {
            custom.set_override(id, style);
            has_overrides = true;
        }
    }

    if has_overrides {
        std::sync::Arc::new(custom)
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
    let mut attrs = Attrs::default();
    for part in parts.iter().skip(2) {
        match *part {
            "bold" => attrs.bold = true,
            "italic" => attrs.italic = true,
            "underline" => attrs.underline = true,
            "dim" => attrs.dim = true,
            _ => {}
        }
    }
    Some(Style { fg, bg, attrs })
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
            style.fg = Color::Ansi(n as u8);
        }
    }
}
