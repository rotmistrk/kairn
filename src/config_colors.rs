//! Color configuration loading from Tcl variables.

use rusticle::interpreter::Interpreter;
use txv_core::cell::{Color, Style};

use crate::app_palette::AppPalette;

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

fn apply_fg(interp: &Interpreter, var: &str, style: &mut Style) {
    if let Some(val) = interp.get_var(var) {
        if let Ok(n) = val.as_int() {
            style.fg = Color::Ansi(n as u8);
        }
    }
}
