use ratatui::layout::Rect;

use super::{LayoutMode, PanelSizes};

/// Computed panel rectangles for a given frame.
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub tree: Option<Rect>,
    pub main: Rect,
    pub interactive: Rect,
}

impl LayoutConstraints {
    pub fn compute(area: Rect, mode: LayoutMode, sizes: &PanelSizes) -> Self {
        let tree_w = sizes.tree_width.min(area.width / 3);
        let has_tree = tree_w > 0;

        match mode {
            LayoutMode::Wide => Self::wide(area, tree_w, has_tree, sizes),
            LayoutMode::TallRight => Self::tall_right(area, tree_w, has_tree, sizes),
            LayoutMode::TallBottom => Self::tall_bottom(area, tree_w, has_tree, sizes),
        }
    }

    fn wide(area: Rect, tree_w: u16, has_tree: bool, sizes: &PanelSizes) -> Self {
        let content_x = if has_tree { area.x + tree_w } else { area.x };
        let content_w = area.width.saturating_sub(if has_tree { tree_w } else { 0 });
        let int_w = sizes.interactive_size.min(content_w / 2);
        let main_w = content_w.saturating_sub(int_w);
        Self {
            tree: tree_rect(area, tree_w, has_tree, area.height),
            main: Rect::new(content_x, area.y, main_w, area.height),
            interactive: Rect::new(content_x + main_w, area.y, int_w, area.height),
        }
    }

    fn tall_right(area: Rect, tree_w: u16, has_tree: bool, sizes: &PanelSizes) -> Self {
        let content_x = if has_tree { area.x + tree_w } else { area.x };
        let content_w = area.width.saturating_sub(if has_tree { tree_w } else { 0 });
        let int_h = sizes.interactive_size.min(area.height / 2);
        let main_h = area.height.saturating_sub(int_h);
        Self {
            tree: tree_rect(area, tree_w, has_tree, area.height),
            main: Rect::new(content_x, area.y, content_w, main_h),
            interactive: Rect::new(content_x, area.y + main_h, content_w, int_h),
        }
    }

    fn tall_bottom(area: Rect, tree_w: u16, has_tree: bool, sizes: &PanelSizes) -> Self {
        let int_h = sizes.interactive_size.min(area.height / 2);
        let top_h = area.height.saturating_sub(int_h);
        let content_x = if has_tree { area.x + tree_w } else { area.x };
        let content_w = area.width.saturating_sub(if has_tree { tree_w } else { 0 });
        Self {
            // Tree only spans top portion, not overlapping bottom panel
            tree: tree_rect(area, tree_w, has_tree, top_h),
            main: Rect::new(content_x, area.y, content_w, top_h),
            interactive: Rect::new(area.x, area.y + top_h, area.width, int_h),
        }
    }
}

fn tree_rect(area: Rect, tree_w: u16, has_tree: bool, height: u16) -> Option<Rect> {
    if has_tree {
        Some(Rect::new(area.x, area.y, tree_w, height))
    } else {
        None
    }
}
