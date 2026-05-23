//! Compatibility shim — re-exports from `crate::slots`.

pub use crate::slots::{
    find_view_mut, focus_view_mut, insert_tab, slot_from, Desktop, LayoutMode, SlotId, PANEL_COUNT,
};
