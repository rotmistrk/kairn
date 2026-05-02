//! Expandable/collapsible tree with cursor navigation.

use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent};
use txv::surface::Surface;

use crate::scroll_view::ScrollView;
use crate::widget::{EventResult, Widget, WidgetAction};

/// Data source for a tree view.
pub trait TreeData {
    /// Node identifier type.
    type NodeId: Clone + Eq + std::hash::Hash + std::fmt::Debug;

    /// Root-level nodes.
    fn root_nodes(&self) -> Vec<Self::NodeId>;

    /// Children of a node.
    fn children(&self, id: &Self::NodeId) -> Vec<Self::NodeId>;

    /// Whether a node has children.
    fn has_children(&self, id: &Self::NodeId) -> bool;

    /// Render a node into a one-row surface.
    fn render_node(
        &self,
        id: &Self::NodeId,
        surface: &mut Surface<'_>,
        depth: usize,
        expanded: bool,
        selected: bool,
    );
}

/// A flattened entry in the visible tree.
struct FlatNode<Id> {
    id: Id,
    depth: usize,
}

/// Expandable/collapsible tree with cursor and scroll.
pub struct TreeView<D: TreeData> {
    data: D,
    expanded: HashSet<D::NodeId>,
    flat_nodes: Vec<FlatNode<D::NodeId>>,
    cursor: usize,
    scroll: ScrollView,
}

impl<D: TreeData> TreeView<D> {
    /// Create a new tree view with the given data source.
    pub fn new(data: D) -> Self {
        let mut tv = Self {
            data,
            expanded: HashSet::new(),
            flat_nodes: Vec::new(),
            cursor: 0,
            scroll: ScrollView::new(),
        };
        tv.rebuild_flat();
        tv
    }

    /// Rebuild the flat node list from the tree structure.
    pub fn rebuild_flat(&mut self) {
        self.flat_nodes.clear();
        let roots = self.data.root_nodes();
        for root in &roots {
            self.flatten_node(root, 0);
        }
        self.scroll.set_content_size(self.flat_nodes.len(), 0);
        if self.cursor >= self.flat_nodes.len() {
            self.cursor = self.flat_nodes.len().saturating_sub(1);
        }
    }

    fn flatten_node(&mut self, id: &D::NodeId, depth: usize) {
        self.flat_nodes.push(FlatNode {
            id: id.clone(),
            depth,
        });
        if self.expanded.contains(id) {
            let children = self.data.children(id);
            for child in &children {
                self.flatten_node(child, depth + 1);
            }
        }
    }

    /// Get the currently selected node ID.
    pub fn selected_node(&self) -> Option<&D::NodeId> {
        self.flat_nodes.get(self.cursor).map(|n| &n.id)
    }

    /// Replace the data source. Clears expansion and resets cursor.
    pub fn set_data(&mut self, data: D) {
        self.data = data;
        self.expanded.clear();
        self.cursor = 0;
        self.rebuild_flat();
    }

    /// Expand a node.
    pub fn expand(&mut self, id: &D::NodeId) {
        if self.data.has_children(id) {
            self.expanded.insert(id.clone());
            self.rebuild_flat();
        }
    }

    /// Collapse a node.
    pub fn collapse(&mut self, id: &D::NodeId) {
        if self.expanded.remove(id) {
            self.rebuild_flat();
        }
    }

    /// Toggle expand/collapse.
    pub fn toggle(&mut self, id: &D::NodeId) {
        if self.expanded.contains(id) {
            self.collapse(id);
        } else {
            self.expand(id);
        }
    }

    /// Get a reference to the data source.
    pub fn data(&self) -> &D {
        &self.data
    }

    fn cursor_node(&self) -> Option<&FlatNode<D::NodeId>> {
        self.flat_nodes.get(self.cursor)
    }

    fn find_parent_index(&self) -> Option<usize> {
        let node = self.cursor_node()?;
        if node.depth == 0 {
            return None;
        }
        let target_depth = node.depth - 1;
        (0..self.cursor)
            .rev()
            .find(|&i| self.flat_nodes[i].depth == target_depth)
    }

    fn handle_right(&mut self) -> EventResult {
        let Some(node) = self.cursor_node() else {
            return EventResult::Ignored;
        };
        let id = node.id.clone();
        if self.data.has_children(&id) {
            if self.expanded.contains(&id) {
                // Move to first child
                if self.cursor + 1 < self.flat_nodes.len() {
                    self.cursor += 1;
                }
            } else {
                self.expand(&id);
            }
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn handle_left(&mut self) -> EventResult {
        let Some(node) = self.cursor_node() else {
            return EventResult::Ignored;
        };
        let id = node.id.clone();
        if self.expanded.contains(&id) {
            self.collapse(&id);
            EventResult::Consumed
        } else if let Some(parent_idx) = self.find_parent_index() {
            self.cursor = parent_idx;
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }
}

impl<D: TreeData> Widget for TreeView<D> {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let h = surface.height();
        let w = surface.width();
        let range = self.scroll.visible_range(h);

        for (row_idx, flat_idx) in range.enumerate() {
            let node = &self.flat_nodes[flat_idx];
            let is_selected = flat_idx == self.cursor;
            let is_expanded = self.expanded.contains(&node.id);
            let mut row_surface = surface.sub(0, row_idx as u16, w, 1);
            self.data.render_node(
                &node.id,
                &mut row_surface,
                node.depth,
                is_expanded,
                is_selected,
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if self.flat_nodes.is_empty() {
            return match key.code {
                KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
                _ => EventResult::Ignored,
            };
        }
        match key.code {
            KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                self.scroll.ensure_visible(self.cursor, 0);
                EventResult::Consumed
            }
            KeyCode::Down => {
                let max = self.flat_nodes.len().saturating_sub(1);
                self.cursor = (self.cursor + 1).min(max);
                self.scroll.ensure_visible(self.cursor, 0);
                EventResult::Consumed
            }
            KeyCode::Right => self.handle_right(),
            KeyCode::Left => self.handle_left(),
            KeyCode::Home => {
                self.cursor = 0;
                self.scroll.ensure_visible(0, 0);
                EventResult::Consumed
            }
            KeyCode::End => {
                self.cursor = self.flat_nodes.len().saturating_sub(1);
                self.scroll.ensure_visible(self.cursor, 0);
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(node) = self.cursor_node() {
                    let s = format!("{:?}", node.id);
                    EventResult::Action(WidgetAction::Selected(s))
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use txv::cell::Style;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct TestId(String);

    struct TestTree {
        nodes: Vec<(TestId, Vec<TestId>)>,
    }

    impl TreeData for TestTree {
        type NodeId = TestId;

        fn root_nodes(&self) -> Vec<TestId> {
            self.nodes
                .iter()
                .filter(|(id, _)| !self.nodes.iter().any(|(_, ch)| ch.contains(id)))
                .map(|(id, _)| id.clone())
                .collect()
        }

        fn children(&self, id: &TestId) -> Vec<TestId> {
            self.nodes
                .iter()
                .find(|(nid, _)| nid == id)
                .map(|(_, ch)| ch.clone())
                .unwrap_or_default()
        }

        fn has_children(&self, id: &TestId) -> bool {
            self.nodes
                .iter()
                .any(|(nid, ch)| nid == id && !ch.is_empty())
        }

        fn render_node(
            &self,
            id: &TestId,
            surface: &mut Surface<'_>,
            depth: usize,
            expanded: bool,
            selected: bool,
        ) {
            let indent = "  ".repeat(depth);
            let marker = if self.has_children(id) {
                if expanded {
                    "▾"
                } else {
                    "▸"
                }
            } else {
                " "
            };
            let sel = if selected { ">" } else { " " };
            let label = format!("{sel}{indent}{marker}{}", id.0);
            surface.print(0, 0, &label, Style::default());
        }
    }

    fn make_tree() -> TestTree {
        TestTree {
            nodes: vec![
                (
                    TestId("root".into()),
                    vec![TestId("child1".into()), TestId("child2".into())],
                ),
                (TestId("child1".into()), vec![TestId("leaf1".into())]),
                (TestId("child2".into()), vec![]),
                (TestId("leaf1".into()), vec![]),
            ],
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn new_shows_roots() {
        let tv = TreeView::new(make_tree());
        assert_eq!(tv.flat_nodes.len(), 1); // only root visible
        assert_eq!(tv.selected_node(), Some(&TestId("root".into())));
    }

    #[test]
    fn expand_shows_children() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        assert_eq!(tv.flat_nodes.len(), 3); // root + child1 + child2
    }

    #[test]
    fn collapse_hides_children() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        tv.collapse(&TestId("root".into()));
        assert_eq!(tv.flat_nodes.len(), 1);
    }

    #[test]
    fn toggle_expand_collapse() {
        let mut tv = TreeView::new(make_tree());
        tv.toggle(&TestId("root".into()));
        assert_eq!(tv.flat_nodes.len(), 3);
        tv.toggle(&TestId("root".into()));
        assert_eq!(tv.flat_nodes.len(), 1);
    }

    #[test]
    fn cursor_navigation() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        // root, child1, child2
        assert_eq!(tv.cursor, 0);
        tv.handle_key(key(KeyCode::Down));
        assert_eq!(tv.cursor, 1);
        assert_eq!(tv.selected_node(), Some(&TestId("child1".into())));
        tv.handle_key(key(KeyCode::Down));
        assert_eq!(tv.cursor, 2);
        tv.handle_key(key(KeyCode::Down)); // clamped
        assert_eq!(tv.cursor, 2);
        tv.handle_key(key(KeyCode::Up));
        assert_eq!(tv.cursor, 1);
    }

    #[test]
    fn right_expands() {
        let mut tv = TreeView::new(make_tree());
        // cursor on root, right should expand
        tv.handle_key(key(KeyCode::Right));
        assert_eq!(tv.flat_nodes.len(), 3);
    }

    #[test]
    fn right_on_expanded_moves_to_child() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        tv.handle_key(key(KeyCode::Right));
        assert_eq!(tv.cursor, 1); // moved to child1
    }

    #[test]
    fn left_collapses() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        // cursor on root (expanded), left should collapse
        tv.handle_key(key(KeyCode::Left));
        assert_eq!(tv.flat_nodes.len(), 1);
    }

    #[test]
    fn left_on_child_goes_to_parent() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        tv.cursor = 1; // child1
        tv.handle_key(key(KeyCode::Left));
        assert_eq!(tv.cursor, 0); // back to root
    }

    #[test]
    fn enter_selects() {
        let mut tv = TreeView::new(make_tree());
        let result = tv.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Selected(s)) if s.contains("root")
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut tv = TreeView::new(make_tree());
        let result = tv.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn empty_tree() {
        let tv = TreeView::new(TestTree { nodes: vec![] });
        assert!(tv.flat_nodes.is_empty());
        assert!(tv.selected_node().is_none());
    }

    #[test]
    fn set_data_resets() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        tv.cursor = 2;
        tv.set_data(TestTree { nodes: vec![] });
        assert_eq!(tv.cursor, 0);
        assert!(tv.flat_nodes.is_empty());
    }

    #[test]
    fn deep_expand() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        tv.expand(&TestId("child1".into()));
        // root, child1, leaf1, child2
        assert_eq!(tv.flat_nodes.len(), 4);
        assert_eq!(tv.flat_nodes[2].id, TestId("leaf1".into()));
        assert_eq!(tv.flat_nodes[2].depth, 2);
    }

    #[test]
    fn home_end() {
        let mut tv = TreeView::new(make_tree());
        tv.expand(&TestId("root".into()));
        tv.handle_key(key(KeyCode::End));
        assert_eq!(tv.cursor, 2);
        tv.handle_key(key(KeyCode::Home));
        assert_eq!(tv.cursor, 0);
    }
}
