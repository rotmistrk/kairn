//! Command dispatch for Desktop — handles kairn-specific panel commands.

use txv_core::prelude::*;
use txv_widgets::tiled_workspace::types::SplitDir;

use super::{Desktop, SlotId};
use crate::commands::*;

impl Desktop {
    pub(super) fn handle_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> HandleResult {
        if let Some(r) = self.handle_focus_command(id) {
            return r;
        }
        if let Some(r) = self.handle_tab_command(id, data) {
            return r;
        }
        self.handle_resize_command(id)
    }

    fn handle_focus_command(&mut self, id: CommandId) -> Option<HandleResult> {
        match id {
            CM_FOCUS_LEFT => self.workspace.focus_panel(SlotId::Left as usize),
            CM_FOCUS_CENTER => self.workspace.focus_panel(SlotId::Center as usize),
            CM_FOCUS_RIGHT => self.workspace.focus_panel(SlotId::Right as usize),
            CM_FOCUS_BOTTOM => self.workspace.focus_panel(SlotId::Bottom as usize),
            CM_FOCUS_PREV => self.cycle_focus(-1),
            CM_FOCUS_NEXT => self.cycle_focus(1),
            CM_ZOOM_TOGGLE => self.workspace.toggle_zoom(),
            _ => return None,
        }
        Some(HandleResult::Consumed)
    }

    fn handle_tab_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> Option<HandleResult> {
        let focused = self.workspace.focused_panel();
        match id {
            CM_TAB_NEXT => {
                if let Some(p) = self.workspace.panel_mut(focused) {
                    p.tab_next();
                }
            }
            CM_TAB_PREV => {
                if let Some(p) = self.workspace.panel_mut(focused) {
                    p.tab_prev();
                }
            }
            CM_FOCUS_TAB => self.dispatch_focus_tab(data),
            CM_TAB_CLOSE => self.dispatch_tab_close(),
            CM_TAB_DROPDOWN | CM_TAB_DROPDOWN_UP | CM_TAB_DROPDOWN_DOWN => {
                self.dispatch_dropdown(id);
            }
            _ => return None,
        }
        Some(HandleResult::Consumed)
    }

    fn handle_resize_command(&mut self, id: CommandId) -> HandleResult {
        match id {
            CM_PANEL_GROW => self.workspace.resize_panel(SplitDir::Horizontal, 2),
            CM_PANEL_SHRINK => self.workspace.resize_panel(SplitDir::Horizontal, -2),
            CM_PANEL_GROW_V => self.workspace.resize_panel(SplitDir::Vertical, 2),
            CM_PANEL_SHRINK_V => self.workspace.resize_panel(SplitDir::Vertical, -2),
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }

    fn cycle_focus(&mut self, dir: i32) {
        if dir > 0 {
            self.workspace.focus_next_visible();
        } else {
            self.workspace.focus_prev_visible();
        }
        if self.workspace.is_zoomed() {
            self.workspace.set_zoomed(Some(self.workspace.focused_panel()));
        }
    }

    fn dispatch_focus_tab(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let Some(idx) = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) else {
            return;
        };
        let focused = self.workspace.focused_panel();
        if *idx == 0 {
            if let Some(panel) = self.workspace.panel_mut(focused) {
                if panel.dropdown_open() {
                    panel.close_dropdown();
                } else if panel.tab_count() > 1 {
                    panel.open_dropdown();
                }
            }
        } else if let Some(panel) = self.workspace.panel_mut(focused) {
            panel.activate_by_number(*idx as usize);
        }
    }

    fn dispatch_tab_close(&mut self) {
        let focused = self.workspace.focused_panel();
        let title = self
            .workspace
            .panel(focused)
            .and_then(|p| p.active_title().map(String::from));
        if let Some(panel) = self.workspace.panel_mut(focused) {
            panel.close_active();
        }
        self.workspace.put_command(
            CM_FILE_CLOSED,
            title.map(|t| Box::new(t) as Box<dyn std::any::Any + Send>),
        );
    }

    fn dispatch_dropdown(&mut self, id: CommandId) {
        let focused = self.workspace.focused_panel();
        if let Some(panel) = self.workspace.panel_mut(focused) {
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
}
