//! TreeTableSource implementation for TodoTreeData.

use txv_core::cell::Style;
use txv_widgets::tree_table_source::{ColAlign, TreeTableSource};
use txv_widgets::tree_view::TreeData;

use super::data::TodoTreeData;
use super::model::{self, Completion, WorkStatus};
use crate::app_palette::app_palette;

impl TreeTableSource for TodoTreeData {
    fn visible_count(&self) -> usize {
        TreeData::visible_count(self)
    }

    fn label(&self, row: usize) -> &str {
        let id = self.visible_id(row);
        TreeData::label(self, id)
    }

    fn depth(&self, row: usize) -> usize {
        let id = self.visible_id(row);
        TreeData::depth(self, id)
    }

    fn is_expandable(&self, row: usize) -> bool {
        let id = self.visible_id(row);
        TreeData::is_expandable(self, id)
    }

    fn is_expanded(&self, row: usize) -> bool {
        let id = self.visible_id(row);
        TreeData::is_expanded(self, id)
    }

    fn toggle(&mut self, row: usize) {
        let id = self.visible_id(row);
        TreeData::toggle(self, id);
    }

    fn style(&self, row: usize) -> Style {
        let id = self.visible_id(row);
        TreeData::style(self, id)
    }

    fn highlight_positions(&self, row: usize) -> Option<&[usize]> {
        let id = self.visible_id(row);
        TreeData::highlight_positions(self, id)
    }

    fn filter_status(&self) -> Option<&str> {
        TreeData::filter_status(self)
    }

    fn column_count(&self) -> usize {
        if self.show_loe {
            2
        } else {
            1
        }
    }

    fn cell(&self, row: usize, col: usize) -> &str {
        let id = self.visible_id(row);
        match col {
            0 => self.badge_at(id),
            1 => self.loe_strings.get(id).map(|s| s.as_str()).unwrap_or("  "),
            _ => "",
        }
    }

    fn cell_style(&self, row: usize, _col: usize) -> Style {
        let id = self.visible_id(row);
        let app = app_palette();
        let Some(item) = self.item_at(id) else {
            return app.todo().normal();
        };
        if item.completed == Completion::Done {
            app.todo().done()
        } else {
            app.todo().normal()
        }
    }

    fn column_align(&self, col: usize) -> ColAlign {
        match col {
            1 => ColAlign::Right,
            _ => ColAlign::Left,
        }
    }
}

impl TodoTreeData {
    /// Compute the 3-char badge string for a node.
    pub(crate) fn badge_at(&self, id: usize) -> &str {
        // We store precomputed badges in self.badges indexed by visible row.
        // But since cell() gives us row and we convert to id, we need badges by node id.
        // Actually, let's use the badges vec indexed by node index.
        self.badges.get(id).map(|s| s.as_str()).unwrap_or("○  ")
    }

    /// Rebuild badge strings for all nodes.
    pub(super) fn rebuild_badges(&mut self) {
        self.badges.clear();
        self.loe_strings.clear();
        for i in 0..self.node_count() {
            self.badges.push(Self::compute_badge(self, i));
            self.loe_strings.push(Self::compute_loe(self, i));
        }
    }

    fn compute_loe(&self, id: usize) -> String {
        let Some(item) = self.item_at(id) else {
            return "  ".to_string();
        };
        let collapsed = self.is_expandable_node(id) && !self.is_expanded_node(id);
        let effort = if collapsed {
            model::effective_effort(item)
        } else {
            u16::from(item.effort.unwrap_or(0))
        };
        if effort == 0 {
            "  ".to_string()
        } else {
            format!("{:>2}", effort)
        }
    }

    fn compute_badge(&self, id: usize) -> String {
        let Some(item) = self.item_at(id) else {
            return "○  ".to_string();
        };
        let collapsed = self.is_expandable_node(id) && !self.is_expanded_node(id);
        let status = Self::status_char(item, collapsed);
        let prio = if collapsed {
            model::effective_priority(item)
        } else {
            item.priority.unwrap_or(0)
        };
        let prio_ch = Self::prio_char(prio);
        let has_notes = if collapsed {
            model::effective_has_notes(item)
        } else {
            !item.note.is_empty()
        };
        let notes_ch = if has_notes {
            '♪'
        } else {
            ' '
        };
        format!("{status}{prio_ch}{notes_ch}")
    }

    fn status_char(item: &model::TodoItem, collapsed: bool) -> char {
        if item.completed == Completion::Done {
            return '✓';
        }
        if collapsed && model::effective_in_progress(item) {
            return '▶';
        }
        if collapsed && model::effective_paused(item) {
            return '‖';
        }
        match item.work_status {
            WorkStatus::InProgress => '▶',
            WorkStatus::Paused => '‖',
            _ if item.completed == Completion::Partial => '◐',
            _ => '○',
        }
    }

    fn prio_char(prio: u8) -> char {
        match prio {
            1 => '⠁',
            2 => '⠃',
            3 => '⠇',
            4 => '⡇',
            5 => '⣇',
            6 => '⣧',
            7 => '⣷',
            8.. => '⣿',
            _ => ' ',
        }
    }

    /// Number of flat nodes.
    pub(super) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check expandable by node index (not visible row).
    fn is_expandable_node(&self, id: usize) -> bool {
        self.nodes.get(id).is_some_and(|n| n.expandable)
    }

    /// Check expanded by node index (not visible row).
    fn is_expanded_node(&self, id: usize) -> bool {
        self.nodes.get(id).is_some_and(|n| n.expanded)
    }
}
