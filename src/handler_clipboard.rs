//! Clipboard command handlers for InputLine integration.

use txv_core::program::CommandContext;
use txv_widgets::input_line::{CM_CLIPBOARD_PASTE, CM_COPY_TO_CLIPBOARD, CM_PASTE_REQUEST};

use crate::clipboard::{copy_to_clipboard, paste_from_clipboard};

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
