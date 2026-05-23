//! Command dispatch for LayoutGroup.

use txv_core::prelude::*;

use super::{LayoutGroup, SlotId};
use crate::commands::*;

impl LayoutGroup {
    pub(super) fn handle_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> HandleResult {
        match id {
            CM_FOCUS_LEFT => {
                self.focus_slot(SlotId::Left);
                HandleResult::Consumed
            }
            CM_FOCUS_CENTER => {
                self.focus_slot(SlotId::Center);
                HandleResult::Consumed
            }
            CM_FOCUS_RIGHT => {
                self.focus_slot(SlotId::Right);
                HandleResult::Consumed
            }
            CM_FOCUS_BOTTOM => {
                self.focus_slot(SlotId::Bottom);
                HandleResult::Consumed
            }
            CM_FOCUS_PREV => {
                self.cycle_focus(-1);
                HandleResult::Consumed
            }
            CM_FOCUS_NEXT => {
                self.cycle_focus(1);
                HandleResult::Consumed
            }
            CM_ZOOM_TOGGLE => {
                self.toggle_zoom();
                HandleResult::Consumed
            }
            CM_TAB_NEXT => {
                self.panel_mut(self.focused_slot()).tab_next();
                HandleResult::Consumed
            }
            CM_TAB_PREV => {
                self.panel_mut(self.focused_slot()).tab_prev();
                HandleResult::Consumed
            }
            CM_FOCUS_TAB => {
                if let Some(idx) = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) {
                    let slot = self.focused_slot();
                    if *idx == 0 {
                        // M-0: open/close dropdown
                        let panel = self.panel_mut(slot);
                        if panel.dropdown_open() {
                            panel.close_dropdown();
                        } else if panel.tab_count() > 1 {
                            panel.open_dropdown();
                        }
                        self.group.mark_dirty();
                    } else {
                        // M-N: activate by number (mode-aware)
                        let panel = self.panel_mut(slot);
                        panel.activate_by_number(*idx as usize);
                    }
                }
                HandleResult::Consumed
            }
            CM_TAB_CLOSE => {
                let slot = self.focused_slot();
                let title = self.panel(slot).active_title().map(String::from);
                self.panel_mut(slot).close_active();
                self.group.put_command(
                    CM_FILE_CLOSED,
                    title.map(|t| Box::new(t) as Box<dyn std::any::Any + Send>),
                );
                HandleResult::Consumed
            }
            CM_TAB_DROPDOWN | CM_TAB_DROPDOWN_UP | CM_TAB_DROPDOWN_DOWN => {
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
                self.group.mark_dirty();
                HandleResult::Consumed
            }
            CM_PANEL_GROW => {
                self.resize_focused(2);
                HandleResult::Consumed
            }
            CM_PANEL_SHRINK => {
                self.resize_focused(-2);
                HandleResult::Consumed
            }
            CM_PANEL_GROW_V => {
                self.resize_vertical(2);
                HandleResult::Consumed
            }
            CM_PANEL_SHRINK_V => {
                self.resize_vertical(-2);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    pub(super) fn resize_focused(&mut self, delta: i16) {
        match self.group.focused_index() {
            0 => {
                self.left_width = (self.left_width as i16 + delta).max(8) as u16;
            }
            2 => {
                self.right_width = (self.right_width as i16 + delta).max(8) as u16;
            }
            _ => return,
        }
        self.recompute_bounds();
    }

    pub(super) fn resize_vertical(&mut self, delta: i16) {
        match self.group.focused_index() {
            2 => {
                self.right_height = (self.right_height as i16 + delta).max(3) as u16;
            }
            3 => {
                self.bottom_height = (self.bottom_height as i16 + delta).max(3) as u16;
            }
            _ => return,
        }
        self.recompute_bounds();
    }
}
