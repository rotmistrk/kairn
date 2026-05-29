//! Sidekick (completion popup) overlay management.

use txv_core::geometry::Rect;
use txv_core::prelude::View;
use txv_core::program::CommandContext;
use txv_widgets::sidekick::{
    SidekickShow, SidekickUpdate, SidekickView, CM_SIDEKICK_HIDE, CM_SIDEKICK_SHOW, CM_SIDEKICK_UPDATE,
};

/// Intercept sidekick commands and manage the overlay.
/// Returns true if the command was handled.
pub(crate) fn intercept_sidekick(ctx: &mut CommandContext) -> bool {
    if ctx.command == CM_SIDEKICK_SHOW {
        show_sidekick(ctx);
        return true;
    }
    if ctx.command == CM_SIDEKICK_UPDATE {
        update_sidekick(ctx);
        return true;
    }
    if ctx.command == CM_SIDEKICK_HIDE {
        *ctx.overlay = None;
        return true;
    }
    false
}

fn show_sidekick(ctx: &mut CommandContext) {
    let Some(data) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<SidekickShow>()) else {
        return;
    };
    let mut view = SidekickView::new();
    let screen_h = ctx.desktop.bounds().h;
    let popup_h = data.rect.h.min(8);
    let y = screen_h.saturating_sub(popup_h);
    view.set_bounds(Rect::new(data.rect.x, y, data.rect.w, popup_h));
    view.set_items(data.items.clone(), data.selected);
    *ctx.overlay = Some(Box::new(view));
}

fn update_sidekick(ctx: &mut CommandContext) {
    let Some(data) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<SidekickUpdate>()) else {
        return;
    };
    let Some(overlay) = ctx.overlay.as_mut() else {
        return;
    };
    if let Some(sk) = overlay.as_any_mut().and_then(|a| a.downcast_mut::<SidekickView>()) {
        sk.set_items(data.items.clone(), data.selected);
    }
}
