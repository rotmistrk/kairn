//! Command dispatch for Desktop — handles kairn-specific panel commands.

use txv_core::prelude::*;

use super::{Desktop, SlotId};
use crate::commands::*;

impl Desktop {
    pub(crate) fn handle_command(
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
            CM_FOCUS_LEFT => self.focus_slot(SlotId::Left),
            CM_FOCUS_CENTER => self.focus_slot(SlotId::Center),
            CM_FOCUS_RIGHT => self.focus_slot(SlotId::Right),
            CM_FOCUS_BOTTOM => self.focus_slot(SlotId::Bottom),
            CM_FOCUS_PREV => self.cycle_focus(-1),
            CM_FOCUS_NEXT => self.cycle_focus(1),
            CM_ZOOM_TOGGLE => self.toggle_zoom(),
            _ => return None,
        }
        Some(HandleResult::Consumed)
    }

    fn handle_tab_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> Option<HandleResult> {
        match id {
            CM_TAB_NEXT => self.panel_mut(self.focused_slot()).tab_next(),
            CM_TAB_PREV => self.panel_mut(self.focused_slot()).tab_prev(),
            CM_FOCUS_TAB => self.dispatch_focus_tab(data),
            CM_TAB_CLOSE => self.dispatch_tab_close(),
            CM_TAB_DROPDOWN | CM_TAB_DROPDOWN_UP | CM_TAB_DROPDOWN_DOWN => {
                self.dispatch_dropdown(id);
            }
            CM_PANEL_GROW | CM_PANEL_SHRINK | CM_PANEL_GROW_V | CM_PANEL_SHRINK_V => return None,
            _ => return None,
        }
        Some(HandleResult::Consumed)
    }

    fn handle_resize_command(&mut self, id: CommandId) -> HandleResult {
        match id {
            CM_PANEL_GROW => self.resize_focused(2),
            CM_PANEL_SHRINK => self.resize_focused(-2),
            CM_PANEL_GROW_V => self.resize_vertical(2),
            CM_PANEL_SHRINK_V => self.resize_vertical(-2),
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }

    fn dispatch_focus_tab(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let Some(idx) = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) else {
            return;
        };
        let slot = self.focused_slot();
        if *idx == 0 {
            let panel = self.panel_mut(slot);
            if panel.dropdown_open() {
                panel.close_dropdown();
            } else if panel.tab_count() > 1 {
                panel.open_dropdown();
            }
        } else {
            self.panel_mut(slot).activate_by_number(*idx as usize);
        }
    }

    fn dispatch_tab_close(&mut self) {
        let slot = self.focused_slot();
        let title = self.panel(slot).active_title().map(String::from);
        self.panel_mut(slot).close_active();
        self.workspace.put_command(
            CM_FILE_CLOSED,
            title.map(|t| Box::new(t) as Box<dyn std::any::Any + Send>),
        );
    }

    fn dispatch_dropdown(&mut self, id: CommandId) {
        let slot = self.focused_slot();
        let panel = self.panel_mut(slot);
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
