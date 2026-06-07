//! Clipboard and diagnostics command handlers.

use txv_core::program::CommandContext;
use txv_widgets::input_line::{CM_CLIPBOARD_PASTE, CM_COPY_TO_CLIPBOARD, CM_PASTE_REQUEST};
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::handler::{downcast_desktop, AppState};
use crate::lsp::diagnostics::Diagnostic;
use crate::slots::SlotId;
use crate::views::problems::ProblemsView;

pub(crate) fn handle_clipboard_commands(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command() {
        CM_COPY_TO_CLIPBOARD => {
            if let Some(text) = ctx.data().as_ref().and_then(|d| d.downcast_ref::<String>()) {
                if let Ok(mut ring) = state.clipboard.lock() {
                    ring.push(text, "input");
                }
            }
        }
        CM_PASTE_REQUEST => {
            if let Ok(mut ring) = state.clipboard.lock() {
                if let Some(text) = ring.paste() {
                    ctx.sink().push_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn update_problems_view(ctx: &mut CommandContext) {
    let (_, data, _, desktop) = ctx.split();
    let Some(data) = data.as_ref() else {
        return;
    };
    let Some((uri, diags)) = data.downcast_ref::<(String, Vec<Diagnostic>)>() else {
        return;
    };
    let Some(desktop) = downcast_desktop(desktop) else {
        return;
    };
    find_problems_view(desktop, |pv| pv.update_diagnostics(uri, diags.clone()));
}

fn find_problems_view(desktop: &mut TiledWorkspace, f: impl FnOnce(&mut ProblemsView)) {
    let Some(panel) = desktop.panel_mut(SlotId::Tools as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        if let Some(pv) = any.downcast_mut::<ProblemsView>() {
            f(pv);
            break;
        }
    }
}
