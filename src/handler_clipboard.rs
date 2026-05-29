//! Clipboard and diagnostics command handlers.

use txv_core::program::CommandContext;
use txv_widgets::input_line::{CM_CLIPBOARD_PASTE, CM_COPY_TO_CLIPBOARD, CM_PASTE_REQUEST};
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::clipboard::{copy_to_clipboard, paste_from_clipboard};
use crate::handler::downcast_desktop;
use crate::lsp::diagnostics::Diagnostic;
use crate::slots::SlotId;
use crate::views::problems::ProblemsView;

pub(crate) fn handle_clipboard_commands(ctx: &mut CommandContext) {
    match ctx.command {
        CM_COPY_TO_CLIPBOARD => {
            if let Some(text) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                if let Err(e) = copy_to_clipboard(text) {
                    log::warn!("clipboard copy: {e}");
                }
            }
        }
        CM_PASTE_REQUEST => match paste_from_clipboard() {
            Ok(text) => ctx.sink.push_command(CM_CLIPBOARD_PASTE, Some(Box::new(text))),
            Err(e) => log::warn!("clipboard paste: {e}"),
        },
        _ => {}
    }
}

pub(crate) fn update_problems_view(ctx: &mut CommandContext) {
    let Some(data) = ctx.data.as_ref() else {
        return;
    };
    let Some((uri, diags)) = data.downcast_ref::<(String, Vec<Diagnostic>)>() else {
        return;
    };
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
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
