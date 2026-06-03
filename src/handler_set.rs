//! Handler for :set options — single source of truth.
//!
//! Each option is defined ONCE in SET_OPTIONS. The handler and the completer
//! both read from this same list. No separate maintenance needed.

use crate::app_state::AppState;
use crate::handler::downcast_desktop;
use crate::slots::SlotId;
use crate::views::tree::FileTreeView;
use txv_core::program::CommandContext;

/// A single :set option definition.
pub struct SetOption {
    /// The option name (e.g. "wrap", "tree.icons").
    pub(crate) name: &'static str,
    /// True if this is a bool toggle (generates "no{name}" automatically).
    pub(crate) is_toggle: bool,
    /// Apply the option. `on` is true for the positive form, false for "no" form.
    pub(crate) apply: fn(&mut CommandContext, &mut AppState, bool),
}

/// Single source of truth for all :set options.
/// Adding an option here automatically makes it available in both
/// the handler AND completion. No other file needs changing.
pub static SET_OPTIONS: &[SetOption] = &[
    SetOption {
        name: "wrap",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults.wrap = on,
    },
    SetOption {
        name: "list",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults.list = on,
    },
    SetOption {
        name: "number",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults.number = on,
    },
    SetOption {
        name: "rainbow",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults.rainbow = on,
    },
    SetOption {
        name: "guides",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults.guides = on,
    },
    SetOption {
        name: "gutter-signs",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults.gutter_signs = on,
    },
    SetOption {
        name: "tree.icons",
        is_toggle: true,
        apply: |ctx, s, on| {
            s.settings.tree_icons = on;
            toggle_tree_icons(ctx.desktop, on);
        },
    },
];

/// Handle :set options — dispatches from the single SET_OPTIONS registry.
pub fn handle_set_global(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(opt) = boxed.downcast_ref::<String>() else {
        return;
    };
    let opt = opt.trim();

    for entry in SET_OPTIONS {
        if opt == entry.name || (entry.is_toggle && opt == format!("{} true", entry.name)) {
            (entry.apply)(ctx, state, true);
            return;
        }
        if entry.is_toggle {
            let no_form = format!("no{}", entry.name);
            if opt == no_form || opt == format!("{} false", entry.name) {
                (entry.apply)(ctx, state, false);
                return;
            }
        }
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
