//! Handler for :set options (wrap, number, list, etc.)

use crate::app_state::AppState;
use txv_core::program::CommandContext;

/// Handle :set options (wrap, number, list, etc.)
pub fn handle_set_global(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(opt) = boxed.downcast_ref::<String>() else {
        return;
    };
    let defaults = &mut state.settings.editor_defaults;
    match opt.as_str() {
        "wrap" => defaults.wrap = true,
        "nowrap" => defaults.wrap = false,
        "list" | "li" => defaults.list = true,
        "nolist" | "noli" => defaults.list = false,
        "number" | "nu" => defaults.number = true,
        "nonumber" | "nonu" => defaults.number = false,
        "rainbow" => defaults.rainbow = true,
        "norainbow" => defaults.rainbow = false,
        "guides" => defaults.guides = true,
        "noguides" => defaults.guides = false,
        "gutter-signs" => defaults.gutter_signs = true,
        "nogutter-signs" => defaults.gutter_signs = false,
        _ => {}
    }
}
