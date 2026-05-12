//! Tab eviction — LRU-based tab limit enforcement.
//!
//! When a slot is at capacity, the least-recently-used tab is evicted.
//! If the LRU tab is dirty, it is activated and its own close prompt is
//! triggered (same flow as `:q`). The new tab is stashed until the close
//! completes or is cancelled.

use txv_core::prelude::*;

use crate::layout_group::SlotId;

/// A pending tab insertion waiting for the LRU tab's close to complete.
pub struct PendingTab {
    pub slot: SlotId,
    pub title: String,
    pub view: Box<dyn View>,
}
