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
        apply: |_, s, on| s.settings.editor_defaults_mut().set_wrap(on),
    },
    SetOption {
        name: "list",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults_mut().set_list(on),
    },
    SetOption {
        name: "number",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults_mut().set_number(on),
    },
    SetOption {
        name: "rainbow",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults_mut().set_rainbow(on),
    },
    SetOption {
        name: "guides",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults_mut().set_guides(on),
    },
    SetOption {
        name: "gutter-signs",
        is_toggle: true,
        apply: |_, s, on| s.settings.editor_defaults_mut().set_gutter_signs(on),
    },
    SetOption {
        name: "tree.icons",
        is_toggle: true,
        apply: |ctx, s, on| {
            s.settings.tree_icons = on;
            toggle_tree_icons(ctx.desktop_mut(), on);
        },
    },
    SetOption {
        name: "tree.connectors",
        is_toggle: true,
        apply: |ctx, s, on| {
            s.settings.tree_connectors = on;
            toggle_tree_connectors(ctx.desktop_mut(), on);
        },
    },
];

/// Handle :set options — dispatches from the single SET_OPTIONS registry.
pub fn handle_set_global(ctx: &mut CommandContext, state: &mut AppState) {
    let opt = {
        let Some(boxed) = ctx.data().as_ref() else {
            return;
        };
        let Some(s) = boxed.downcast_ref::<String>() else {
            return;
        };
        s.trim().to_string()
    };

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
    // Forward unrecognized options to all open editors as per-buffer :set
    apply_set_to_all_editors(ctx, &opt);
}

fn apply_set_to_all_editors(ctx: &mut CommandContext, opt: &str) {
    let Some(d) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let cmd = format!("set {opt}");
    for slot in 0..3 {
        let Some(panel) = d.panel_mut(slot) else {
            continue;
        };
        apply_set_to_panel(panel, &cmd);
    }
}

fn apply_set_to_panel(panel: &mut txv_widgets::tab_panel::TabPanel, cmd: &str) {
    use crate::editor::command::Command;
    use crate::views::editor::EditorView;
    let count = panel.tab_count();
    for i in 0..count {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
            continue;
        };
        ev.editor_mut().execute(Command::ExCommand(cmd.to_string()));
        // Sync delegate settings from editor options
        let cn = ev.editor().options().cursor_normal();
        let ci = ev.editor().options().cursor_insert();
        let cc = ev.editor().options().cursor_command();
        ev.delegate_mut().settings_mut().set_cursor_normal(cn);
        ev.delegate_mut().settings_mut().set_cursor_insert(ci);
        ev.delegate_mut().settings_mut().set_cursor_command(cc);
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

fn toggle_tree_connectors(desktop: &mut dyn txv_core::view::View, on: bool) {
    use crate::views::struct_view::StructuredView;
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    // Toggle on file tree
    if let Some(panel) = d.panel_mut(SlotId::Left as usize) {
        if let Some(view) = panel.view_at_mut(0) {
            if let Some(tree) = view.as_any_mut().and_then(|a| a.downcast_mut::<FileTreeView>()) {
                tree.inner.set_show_connectors(on);
            }
        }
    }
    // Toggle on active center view if it's a structured view
    if let Some(panel) = d.panel_mut(SlotId::Center as usize) {
        if let Some(view) = panel.active_view_mut() {
            if let Some(sv) = view.as_any_mut().and_then(|a| a.downcast_mut::<StructuredView>()) {
                sv.inner_mut().set_show_connectors(on);
            }
        }
    }
}
