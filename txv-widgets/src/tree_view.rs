//! TreeView — generic tree widget parameterized by TreeData.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

/// Trait for providing tree data to TreeView.
pub trait TreeData: Send + 'static {
    fn root_count(&self) -> usize;
    fn child_count(&self, id: usize) -> usize;
    fn label(&self, id: usize) -> &str;
    fn is_expandable(&self, id: usize) -> bool;
    fn is_expanded(&self, id: usize) -> bool;
    fn toggle(&mut self, id: usize);
    fn depth(&self, id: usize) -> usize;
    /// Return flat visible row count.
    fn visible_count(&self) -> usize;
    /// Return the node id for a given visible row index.
    fn visible_id(&self, row: usize) -> usize;
}

pub struct TreeView<D: TreeData> {
    state: ViewState,
    pub data: D,
    pub cursor: usize,
    pub scroll: ScrollView,
}

impl<D: TreeData> TreeView<D> {
    pub fn new(data: D) -> Self {
        Self {
            state: ViewState::default(),
            data,
            cursor: 0,
            scroll: ScrollView::new(),
        }
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds.h as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.data.visible_count());
        self.scroll.ensure_visible(self.cursor);
    }
}

impl<D: TreeData> View for TreeView<D> {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let normal = Style::default();
        let selected = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        for row in 0..b.h as usize {
            let idx = self.scroll.offset + row;
            if idx >= self.data.visible_count() {
                break;
            }
            let id = self.data.visible_id(idx);
            let depth = self.data.depth(id);
            let indent = (depth * 2) as u16;
            let marker = if self.data.is_expandable(id) {
                if self.data.is_expanded(id) {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };
            let style = if idx == self.cursor {
                selected
            } else {
                normal
            };
            let y = b.y + row as u16;
            // Clear line
            surface.hline(b.x, y, b.w, ' ', style);
            let x = b.x + indent;
            surface.print(x, y, marker, style);
            surface.print(x + 2, y, self.data.label(id), style);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.sync_scroll();
                        self.state.dirty = true;
                    }
                    HandleResult::Consumed
                }
                KeyCode::Down => {
                    let max = self.data.visible_count().saturating_sub(1);
                    if self.cursor < max {
                        self.cursor += 1;
                        self.sync_scroll();
                        self.state.dirty = true;
                    }
                    HandleResult::Consumed
                }
                KeyCode::Enter | KeyCode::Right => {
                    if self.cursor < self.data.visible_count() {
                        let id = self.data.visible_id(self.cursor);
                        if self.data.is_expandable(id) && !self.data.is_expanded(id) {
                            self.data.toggle(id);
                            self.sync_scroll();
                            self.state.dirty = true;
                        } else {
                            queue.put_command(CM_OK, Some(Box::new(id)));
                        }
                    }
                    HandleResult::Consumed
                }
                KeyCode::Left => {
                    if self.cursor < self.data.visible_count() {
                        let id = self.data.visible_id(self.cursor);
                        if self.data.is_expandable(id) && self.data.is_expanded(id) {
                            self.data.toggle(id);
                            self.sync_scroll();
                            self.state.dirty = true;
                        }
                    }
                    HandleResult::Consumed
                }
                KeyCode::Home => {
                    self.cursor = 0;
                    self.sync_scroll();
                    self.state.dirty = true;
                    HandleResult::Consumed
                }
                KeyCode::End => {
                    self.cursor = self.data.visible_count().saturating_sub(1);
                    self.sync_scroll();
                    self.state.dirty = true;
                    HandleResult::Consumed
                }
                KeyCode::PageDown => {
                    let page = (self.state.bounds.h as usize).saturating_sub(1).max(1);
                    let max = self.data.visible_count().saturating_sub(1);
                    self.cursor = (self.cursor + page).min(max);
                    self.sync_scroll();
                    self.state.dirty = true;
                    HandleResult::Consumed
                }
                KeyCode::PageUp => {
                    let page = (self.state.bounds.h as usize).saturating_sub(1).max(1);
                    self.cursor = self.cursor.saturating_sub(page);
                    self.sync_scroll();
                    self.state.dirty = true;
                    HandleResult::Consumed
                }
                _ => HandleResult::Ignored,
            },
            _ => HandleResult::Ignored,
        }
    }
}
