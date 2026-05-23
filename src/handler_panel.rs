//! Panel focus, tab, resize, and zoom command dispatch.

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::types::SplitDir;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::commands::*;
use crate::handler::downcast_workspace;
use crate::slots::SlotId;

/// Handle panel commands. Returns true if consumed.
pub fn handle_panel_command(ctx: &mut CommandContext) -> bool {
    let Some(ws) = downcast_workspace(ctx.desktop) else {
        return false;
    };
    match ctx.command {
        CM_FOCUS_LEFT => ws.focus_slot(SlotId::Left),
        CM_FOCUS_CENTER => ws.focus_slot(SlotId::Center),
        CM_FOCUS_RIGHT => ws.focus_slot(SlotId::Right),
        CM_FOCUS_BOTTOM => ws.focus_slot(SlotId::Bottom),
        CM_FOCUS_PREV => cycle_focus(&mut ws.ws, -1),
        CM_FOCUS_NEXT => cycle_focus(&mut ws.ws, 1),
        CM_ZOOM_TOGGLE => ws.ws.toggle_zoom(),
        CM_TAB_NEXT => {
            let f = ws.ws.focused_panel();
            if let Some(p) = TiledWorkspace::panel_mut(&mut ws.ws, f) {
                p.tab_next();
            }
        }
        CM_TAB_PREV => {
            let f = ws.ws.focused_panel();
            if let Some(p) = TiledWorkspace::panel_mut(&mut ws.ws, f) {
                p.tab_prev();
            }
        }
        CM_FOCUS_TAB => dispatch_focus_tab(&mut ws.ws, ctx.data),
        CM_TAB_CLOSE => dispatch_tab_close(&mut ws.ws, ctx.sink),
        CM_TAB_DROPDOWN | CM_TAB_DROPDOWN_UP | CM_TAB_DROPDOWN_DOWN => {
            dispatch_dropdown(&mut ws.ws, ctx.command);
        }
        CM_PANEL_GROW => ws.ws.resize_panel(SplitDir::Horizontal, 2),
        CM_PANEL_SHRINK => ws.ws.resize_panel(SplitDir::Horizontal, -2),
        CM_PANEL_GROW_V => ws.ws.resize_panel(SplitDir::Vertical, 2),
        CM_PANEL_SHRINK_V => ws.ws.resize_panel(SplitDir::Vertical, -2),
        _ => return false,
    }
    true
}

fn cycle_focus(ws: &mut TiledWorkspace, dir: i32) {
    if dir > 0 {
        ws.focus_next_visible();
    } else {
        ws.focus_prev_visible();
    }
    if ws.is_zoomed() {
        ws.set_zoomed(Some(ws.focused_panel()));
    }
}

fn dispatch_focus_tab(ws: &mut TiledWorkspace, data: &Option<Box<dyn std::any::Any + Send>>) {
    let Some(idx) = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) else {
        return;
    };
    let focused = ws.focused_panel();
    if *idx == 0 {
        if let Some(panel) = TiledWorkspace::panel_mut(ws, focused) {
            if panel.dropdown_open() {
                panel.close_dropdown();
            } else if panel.tab_count() > 1 {
                panel.open_dropdown();
            }
        }
    } else if let Some(panel) = TiledWorkspace::panel_mut(ws, focused) {
        panel.activate_by_number(*idx as usize);
    }
}

fn dispatch_tab_close(ws: &mut TiledWorkspace, sink: &EventSink) {
    let focused = ws.focused_panel();
    let title = TiledWorkspace::panel(ws, focused).and_then(|p| p.active_title().map(String::from));
    if let Some(panel) = TiledWorkspace::panel_mut(ws, focused) {
        panel.close_active();
    }
    sink.push_command(
        CM_FILE_CLOSED,
        title.map(|t| Box::new(t) as Box<dyn std::any::Any + Send>),
    );
}

fn dispatch_dropdown(ws: &mut TiledWorkspace, id: u16) {
    let focused = ws.focused_panel();
    if let Some(panel) = TiledWorkspace::panel_mut(ws, focused) {
        if panel.dropdown_open() {
            match id {
                CM_TAB_DROPDOWN => panel.close_dropdown(),
                CM_TAB_DROPDOWN_UP => panel.dropdown_move_up(),
                CM_TAB_DROPDOWN_DOWN => panel.dropdown_move_down(),
                _ => {}
            }
        } else if panel.tab_count() > 1 {
            panel.open_dropdown();
        }
    }
}
