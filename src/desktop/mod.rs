//! Compatibility shim — re-exports from `crate::slots`.

pub use crate::slots::{
    active_tab_title, close_tab_by_title, find_view_mut, focus_tab_by_title, focus_view_mut, insert_tab, next_tab_name,
    slot_from, LayoutMode, SlotId, PANEL_COUNT,
};
