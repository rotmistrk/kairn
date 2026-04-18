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
        let content_x = if has_tree { area.x + tree_w } else { area.x };
        let content_w = if has_tree {
            area.width.saturating_sub(tree_w)
        } else {
            area.width
        };

        let tree = if has_tree {
            Some(Rect::new(area.x, area.y, tree_w, area.height))
        } else {
            None
        };

        let (main, interactive) = match mode {
            LayoutMode::Wide => {
                let int_w = sizes.interactive_size.min(content_w / 2);
                let main_w = content_w.saturating_sub(int_w);
                (
                    Rect::new(content_x, area.y, main_w, area.height),
                    Rect::new(content_x + main_w, area.y, int_w, area.height),
                )
            }
            LayoutMode::TallRight => {
                let int_h = sizes.interactive_size.min(area.height / 2);
                let main_h = area.height.saturating_sub(int_h);
                (
                    Rect::new(content_x, area.y, content_w, main_h),
                    Rect::new(content_x, area.y + main_h, content_w, int_h),
                )
            }
            LayoutMode::TallBottom => {
                let int_h = sizes.interactive_size.min(area.height / 2);
                let top_h = area.height.saturating_sub(int_h);
                (
                    Rect::new(content_x, area.y, content_w, top_h),
                    Rect::new(area.x, area.y + top_h, area.width, int_h),
                )
            }
        };

        Self {
            tree,
            main,
            interactive,
        }
    }
}
