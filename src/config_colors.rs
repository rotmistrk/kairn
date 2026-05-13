//! Color configuration loading from Tcl variables.

use rusticle::interpreter::Interpreter;
use txv_core::cell::Color;
use txv_core::palette::PaletteStyle;

use crate::app_palette::AppPalette;

/// Apply color overrides from config to the palette.
pub fn apply_color_config(interp: &Interpreter, palette: &mut AppPalette) {
    // Git colors
    apply_fg(interp, "color.git.added", &mut palette.git.added);
    apply_fg(interp, "color.git.modified", &mut palette.git.modified);
    apply_fg(interp, "color.git.untracked", &mut palette.git.untracked);
    apply_fg(interp, "color.git.ignored", &mut palette.git.ignored);
    apply_fg(interp, "color.git.conflict", &mut palette.git.conflict);
    // Diff
    apply_fg(interp, "color.diff.added", &mut palette.diff.added);
    apply_fg(interp, "color.diff.deleted", &mut palette.diff.deleted);
    apply_fg(interp, "color.diff.fold", &mut palette.diff.fold);
    // Editor
    apply_fg(interp, "color.editor.gutter", &mut palette.editor.gutter);
    apply_fg(interp, "color.editor.list_chars", &mut palette.editor.list_chars);
    // Diagnostics
    apply_fg(interp, "color.diag.error", &mut palette.diag.error);
    apply_fg(interp, "color.diag.warning", &mut palette.diag.warning);
    apply_fg(interp, "color.diag.info", &mut palette.diag.info);
    apply_fg(interp, "color.diag.hint", &mut palette.diag.hint);
    // Tree
    apply_fg(interp, "color.tree.directory", &mut palette.tree.directory);
    // Todo
    apply_fg(interp, "color.todo.normal", &mut palette.todo.normal);
    apply_fg(interp, "color.todo.done", &mut palette.todo.done);
    apply_fg(interp, "color.todo.important", &mut palette.todo.important);
    // Messages
    apply_fg(interp, "color.msg.error", &mut palette.msg.error);
    apply_fg(interp, "color.msg.warning", &mut palette.msg.warning);
    apply_fg(interp, "color.msg.info", &mut palette.msg.info);
    apply_fg(interp, "color.msg.debug", &mut palette.msg.debug);
    // Framework state
    apply_fg(interp, "color.state.error", &mut palette.base.state.error);
    apply_fg(interp, "color.state.warning", &mut palette.base.state.warning);
    apply_fg(interp, "color.state.info", &mut palette.base.state.info);
    apply_fg(interp, "color.state.success", &mut palette.base.state.success);
    apply_fg(interp, "color.state.hint", &mut palette.base.state.hint);
    // Framework base
    apply_fg(interp, "color.base.dim", &mut palette.base.base.dim);
    apply_fg(interp, "color.base.bright", &mut palette.base.base.bright);
}

fn apply_fg(interp: &Interpreter, var: &str, style: &mut PaletteStyle) {
    if let Some(val) = interp.get_var(var) {
        if let Ok(n) = val.as_int() {
            style.fg = Some(Color::Ansi(n as u8));
        }
    }
}
