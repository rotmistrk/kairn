//! StatusBar traits and container.
//!
//! Items implement `ActiveItem` (event handling), `VisibleItem` (rendering),
//! or both. The `StatusBar` container lays them out and routes events.

use crate::cell::{Attrs, Color, Style};
use crate::event::Event;
use crate::geometry::Rect;
use crate::surface::Surface;
use crate::view::{EventQueue, HandleResult, View, ViewOptions, ViewState};

/// Item alignment on the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gravity {
    Left,
    Right,
}

/// An item that translates events into commands.
pub trait ActiveItem: Send {
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult;
    /// Whether this item wants exclusive control of the status bar.
    fn is_exclusive(&self) -> bool {
        false
    }
}

/// An item that renders a label on the status bar.
pub trait VisibleItem: Send {
    fn label(&self) -> &str;
    fn gravity(&self) -> Gravity;
    /// Style for rendering the label. Default is plain.
    fn style(&self) -> crate::cell::Style {
        crate::cell::Style::default()
    }
    /// Called on tick so the item can update its label.
    fn tick(&mut self) {}
}

/// Combined trait for items that are both active and visible.
pub trait StatusBarItem: ActiveItem + VisibleItem {}

/// Blanket impl: anything implementing both traits is a StatusBarItem.
impl<T: ActiveItem + VisibleItem> StatusBarItem for T {}

// --- Internal storage ---

enum ItemSlot {
    Full(Box<dyn StatusBarItem>),
    ActiveOnly(Box<dyn ActiveItem>),
    VisibleOnly(Box<dyn VisibleItem>),
}

/// Composable status bar container. Implements `View`.
pub struct StatusBar {
    items: Vec<ItemSlot>,
    exclusive: Option<usize>,
    state: ViewState,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            exclusive: None,
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
        }
    }

    /// Add an item that is both active and visible.
    pub fn add(&mut self, item: impl StatusBarItem + 'static) {
        self.items.push(ItemSlot::Full(Box::new(item)));
        self.state.dirty = true;
    }

    /// Add an item that handles events but has no visible label.
    pub fn add_active_only(&mut self, item: impl ActiveItem + 'static) {
        self.items.push(ItemSlot::ActiveOnly(Box::new(item)));
    }

    /// Add an item that displays a label but does not handle events.
    pub fn add_visible_only(&mut self, item: impl VisibleItem + 'static) {
        self.items.push(ItemSlot::VisibleOnly(Box::new(item)));
        self.state.dirty = true;
    }

    /// Put an item into exclusive mode (full-width rendering, sole event target).
    pub fn set_exclusive(&mut self, index: usize) {
        if index < self.items.len() {
            self.exclusive = Some(index);
            self.state.dirty = true;
        }
    }

    /// Clear exclusive mode, returning to normal layout.
    pub fn clear_exclusive(&mut self) {
        self.exclusive = None;
        self.state.dirty = true;
    }

    /// Whether an item is currently in exclusive mode.
    pub fn is_exclusive(&self) -> bool {
        self.exclusive.is_some()
    }

    fn tick_items(&mut self) {
        for slot in &mut self.items {
            match slot {
                ItemSlot::Full(item) => item.tick(),
                ItemSlot::VisibleOnly(item) => item.tick(),
                ItemSlot::ActiveOnly(_) => {}
            }
        }
        self.state.dirty = true;
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for StatusBar {
    fn bounds(&self) -> Rect {
        self.state.bounds
    }
    fn set_bounds(&mut self, rect: Rect) {
        self.state.bounds = rect;
        self.state.dirty = true;
    }
    fn options(&self) -> ViewOptions {
        self.state.options
    }
    fn title(&self) -> &str {
        ""
    }
    fn needs_redraw(&self) -> bool {
        self.state.dirty
    }
    fn mark_redrawn(&mut self) {
        self.state.dirty = false;
    }
    fn select(&mut self) {}
    fn unselect(&mut self) {}

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        surface.hline(b.x, b.y, b.w, ' ', style);

        if let Some(idx) = self.exclusive {
            // Exclusive: render only that item full-width
            if let Some(label) = self.visible_label(idx) {
                surface.print_line(b.x, b.y, label, b.w, style);
            }
            return;
        }

        // Normal: left items from left, right items from right
        let mut lx = b.x;
        let right_edge = b.x + b.w;

        struct RightEntry<'a> {
            label: &'a str,
            item_style: Style,
        }
        let mut right_entries: Vec<RightEntry> = Vec::new();

        for slot in &self.items {
            let label = match slot {
                ItemSlot::Full(item) => item.label(),
                ItemSlot::VisibleOnly(item) => item.label(),
                ItemSlot::ActiveOnly(_) => continue,
            };
            if label.is_empty() {
                continue;
            }
            let gravity = match slot {
                ItemSlot::Full(item) => item.gravity(),
                ItemSlot::VisibleOnly(item) => item.gravity(),
                ItemSlot::ActiveOnly(_) => continue,
            };
            let item_style = match slot {
                ItemSlot::Full(item) => item.style(),
                ItemSlot::VisibleOnly(item) => item.style(),
                ItemSlot::ActiveOnly(_) => Style::default(),
            };
            match gravity {
                Gravity::Left => {
                    let text = format!(" {label} ");
                    let tw = text.len() as u16;
                    if lx + tw <= right_edge {
                        surface.print(lx, b.y, &text, style);
                        lx += tw;
                    }
                }
                Gravity::Right => {
                    right_entries.push(RightEntry { label, item_style });
                }
            }
        }

        // Render right-gravity items from right edge
        let mut rx = right_edge;
        for entry in right_entries.iter().rev() {
            let text = format!(" {} ", entry.label);
            let tw = text.len() as u16;
            if rx >= b.x + tw && rx - tw >= lx {
                rx -= tw;
                let s = if entry.item_style.fg != Color::default() {
                    Style {
                        fg: entry.item_style.fg,
                        attrs: style.attrs,
                        ..Style::default()
                    }
                } else {
                    style
                };
                surface.print(rx, b.y, &text, s);
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Tick = event {
            self.tick_items();
            return HandleResult::Ignored;
        }

        if let Some(idx) = self.exclusive {
            let result = self.handle_active(idx, event, queue);
            // Check if item released exclusive
            if !self.item_is_exclusive(idx) {
                self.exclusive = None;
                self.state.dirty = true;
            }
            return result;
        }

        // Route to all active items, first consumed wins
        for i in 0..self.items.len() {
            let result = self.handle_active(i, event, queue);
            if result == HandleResult::Consumed {
                // Check if item claimed exclusive
                if self.item_is_exclusive(i) {
                    self.exclusive = Some(i);
                    self.state.dirty = true;
                }
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}

impl StatusBar {
    fn visible_label(&self, idx: usize) -> Option<&str> {
        match &self.items[idx] {
            ItemSlot::Full(item) => Some(item.label()),
            ItemSlot::VisibleOnly(item) => Some(item.label()),
            ItemSlot::ActiveOnly(_) => None,
        }
    }

    fn handle_active(&mut self, idx: usize, event: &Event, queue: &mut EventQueue) -> HandleResult {
        match &mut self.items[idx] {
            ItemSlot::Full(item) => item.handle(event, queue),
            ItemSlot::ActiveOnly(item) => item.handle(event, queue),
            ItemSlot::VisibleOnly(_) => HandleResult::Ignored,
        }
    }

    fn item_is_exclusive(&self, idx: usize) -> bool {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.is_exclusive(),
            ItemSlot::ActiveOnly(item) => item.is_exclusive(),
            ItemSlot::VisibleOnly(_) => false,
        }
    }
}
