//! Handler for :set options (wrap, number, list, etc.)

use crate::app_state::AppState;
use crate::handler::downcast_desktop;
use crate::slots::SlotId;
use crate::views::tree::FileTreeView;
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
        "tree.icons" | "tree.icons true" => {
            state.settings.tree_icons = true;
            toggle_tree_icons(ctx.desktop, true);
        }
        "tree.icons false" | "notree.icons" => {
            state.settings.tree_icons = false;
            toggle_tree_icons(ctx.desktop, false);
        }
        _ => {}
    }
}

fn toggle_tree_icons(desktop: &mut dyn txv_core::view::View, on: bool) {
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    let Some(panel) = d.panel_mut(SlotId::Left as usize) else {
        return;
    };
    let Some(view) = panel.view_at_mut(0) else {
        return;
    };
    if let Some(tree) = view.as_any_mut().and_then(|a| a.downcast_mut::<FileTreeView>()) {
        tree.set_show_icons(on);
    }
}
