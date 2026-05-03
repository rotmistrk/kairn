//! Panel system: layout, focus management, and panel types.

pub mod bottom_panel;
pub mod control_panel;
pub mod editor_panel;
pub mod editor_view;
pub mod status;
pub mod terminal_panel;
pub mod tree_panel;

// Keep old modules available during transition.
pub mod commit_tree;
pub mod file_tree;
pub mod interactive;
pub mod main_view;

use serde::{Deserialize, Serialize};
use txv::layout::{Constraint, Direction, Rect, Size};

// Re-export old types for backward compat with modules we don't modify.
#[allow(unused_imports)]
pub use self::file_tree::FileTreePanel;
#[allow(unused_imports)]
pub use self::interactive::InteractivePanel;
#[allow(unused_imports)]
pub use self::main_view::MainViewPanel;

/// Which panel currently has keyboard focus (old, kept for compat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPanel {
    #[default]
    Tree,
    Main,
    Interactive,
}

impl FocusedPanel {
    pub fn next(self) -> Self {
        match self {
            Self::Tree => Self::Main,
            Self::Main => Self::Interactive,
            Self::Interactive => Self::Tree,
        }
    }
}

/// Old panel trait (kept for compat with unmodified modules).
pub trait Panel {
    fn render(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect, focused: bool);
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> anyhow::Result<PanelAction>;
}

/// Old panel actions (kept for compat).
#[derive(Debug)]
pub enum PanelAction {
    None,
    OpenFile(String),
    PreviewFile(String),
    SwitchMode,
    SendToKiro(String),
    PreviewCommit(String),
    ExpandLine,
    Yank(String),
    FocusRight,
    FocusLeft,
    PushOutput(crate::buffer::OutputBuffer),
    Quit,
}

/// Adjust scroll offset so `cursor` stays visible within `height` rows.
pub fn adjust_scroll(cursor: usize, current: usize, height: usize) -> usize {
    if height == 0 {
        return 0;
    }
    if cursor < current {
        cursor
    } else if cursor >= current + height {
        cursor - height + 1
    } else {
        current
    }
}

// ── New panel system types ──────────────────────────────────────

/// Top-level panel focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelFocus {
    #[default]
    Editor,
    Bottom,
    Prompt,
}

/// Sub-focus within the editor triptych.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TriptychFocus {
    Tree,
    #[default]
    Editor,
    Control,
}

/// Layout mode for the three-panel arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayoutMode {
    /// Tree | Editor | Control over Bottom. For terminals > 160 cols.
    #[default]
    Wide,
    /// Tree | (Editor over Bottom). Control hidden. 100–160 cols.
    TallRight,
    /// (Tree | Editor) over Bottom. Control hidden. < 100 cols.
    TallBottom,
}

impl LayoutMode {
    /// Cycle to the next layout mode.
    pub fn next(self) -> Self {
        match self {
            Self::Wide => Self::TallRight,
            Self::TallRight => Self::TallBottom,
            Self::TallBottom => Self::Wide,
        }
    }

    /// Auto-select based on terminal width.
    pub fn auto_select(width: u16) -> Self {
        if width > 160 {
            Self::Wide
        } else if width >= 100 {
            Self::TallRight
        } else {
            Self::TallBottom
        }
    }
}

/// Computed rectangles for each panel region.
#[derive(Debug, Clone)]
pub struct PanelRects {
    pub tree: Option<Rect>,
    pub editor: Rect,
    pub control: Option<Rect>,
    pub bottom: Option<Rect>,
    pub status: Rect,
    pub file_tabs: Option<Rect>,
}

/// Persistent layout sizing state.
pub struct LayoutState {
    pub mode: LayoutMode,
    pub screen_w: u16,
    pub screen_h: u16,
    pub tree_width: u16,
    pub control_width: u16,
    pub bottom_height_pct: u16,
    pub tree_visible: bool,
    pub control_visible: bool,
    pub bottom_visible: bool,
    pub manual_mode: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            mode: LayoutMode::Wide,
            screen_w: 80,
            screen_h: 24,
            tree_width: 20,
            control_width: 25,
            bottom_height_pct: 30,
            tree_visible: true,
            control_visible: false,
            bottom_visible: true,
            manual_mode: false,
        }
    }
}

impl LayoutState {
    /// Update screen dimensions and auto-select mode if not manual.
    pub fn resize(&mut self, w: u16, h: u16) {
        self.screen_w = w;
        self.screen_h = h;
        if !self.manual_mode {
            self.mode = LayoutMode::auto_select(w);
        }
    }

    /// Cycle layout mode manually.
    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
        self.manual_mode = true;
    }

    /// Adjust tree width by delta, clamped to 10..=40.
    pub fn resize_tree(&mut self, delta: i16) {
        let v = (self.tree_width as i16).saturating_add(delta);
        self.tree_width = v.clamp(10, 40) as u16;
    }

    /// Adjust bottom height percentage by delta, clamped to 10..=70.
    pub fn resize_bottom(&mut self, delta: i16) {
        let v = (self.bottom_height_pct as i16).saturating_add(delta);
        self.bottom_height_pct = v.clamp(10, 70) as u16;
    }

    /// Compute panel rectangles from current state.
    pub fn compute_rects(&self) -> PanelRects {
        let full = Rect {
            x: 0,
            y: 0,
            w: self.screen_w,
            h: self.screen_h,
        };

        // Status bar always last row.
        let status = Rect {
            x: 0,
            y: self.screen_h.saturating_sub(1),
            w: self.screen_w,
            h: 1,
        };

        let avail_h = self.screen_h.saturating_sub(1); // minus status
        if avail_h == 0 {
            return PanelRects {
                tree: None,
                editor: Rect {
                    x: 0,
                    y: 0,
                    w: self.screen_w,
                    h: 0,
                },
                control: None,
                bottom: None,
                status,
                file_tabs: None,
            };
        }

        let show_control = self.control_visible && self.mode == LayoutMode::Wide;

        match self.mode {
            LayoutMode::Wide => self.compute_wide(full, avail_h, status, show_control),
            LayoutMode::TallRight => self.compute_tall_right(full, avail_h, status),
            LayoutMode::TallBottom => self.compute_tall_bottom(full, avail_h, status),
        }
    }

    fn compute_wide(
        &self,
        _full: Rect,
        avail_h: u16,
        status: Rect,
        show_control: bool,
    ) -> PanelRects {
        // Vertical: top_area | bottom | status
        let (top_h, bottom) = self.split_top_bottom(avail_h);

        let top = Rect {
            x: 0,
            y: 0,
            w: self.screen_w,
            h: top_h,
        };

        // Horizontal: tree | editor | control
        let mut constraints = Vec::new();
        if self.tree_visible {
            constraints.push(Constraint {
                size: Size::Fixed(self.tree_width),
                min: 10,
                max: 40,
            });
        }
        constraints.push(Constraint {
            size: Size::Fill,
            min: 20,
            max: u16::MAX,
        });
        if show_control {
            constraints.push(Constraint {
                size: Size::Fixed(self.control_width),
                min: 15,
                max: 40,
            });
        }

        let cols = top.split(Direction::Horizontal, &constraints);
        let mut idx = 0;
        let tree = if self.tree_visible {
            let r = cols.get(idx).copied();
            idx += 1;
            r
        } else {
            None
        };
        let editor = cols.get(idx).copied().unwrap_or(top);
        idx += 1;
        let control = if show_control {
            cols.get(idx).copied()
        } else {
            None
        };

        PanelRects {
            tree,
            editor,
            control,
            bottom,
            status,
            file_tabs: None,
        }
    }

    fn compute_tall_right(&self, _full: Rect, avail_h: u16, status: Rect) -> PanelRects {
        // Horizontal: tree | right_area
        let mut h_constraints = Vec::new();
        if self.tree_visible {
            h_constraints.push(Constraint {
                size: Size::Fixed(self.tree_width),
                min: 10,
                max: 40,
            });
        }
        h_constraints.push(Constraint {
            size: Size::Fill,
            min: 20,
            max: u16::MAX,
        });

        let top = Rect {
            x: 0,
            y: 0,
            w: self.screen_w,
            h: avail_h,
        };
        let cols = top.split(Direction::Horizontal, &h_constraints);

        let mut idx = 0;
        let tree = if self.tree_visible {
            let r = cols.get(idx).copied();
            idx += 1;
            r
        } else {
            None
        };
        let right = cols.get(idx).copied().unwrap_or(top);

        // Split right vertically: editor | bottom
        let (editor, bottom) = if self.bottom_visible {
            let bot_h = (right.h as u32 * self.bottom_height_pct as u32 / 100) as u16;
            let bot_h = bot_h.clamp(3, right.h.saturating_sub(5));
            let ed_h = right.h.saturating_sub(bot_h);
            (
                Rect {
                    x: right.x,
                    y: right.y,
                    w: right.w,
                    h: ed_h,
                },
                Some(Rect {
                    x: right.x,
                    y: right.y + ed_h,
                    w: right.w,
                    h: bot_h,
                }),
            )
        } else {
            (right, None)
        };

        PanelRects {
            tree,
            editor,
            control: None,
            bottom,
            status,
            file_tabs: None,
        }
    }

    fn compute_tall_bottom(&self, _full: Rect, avail_h: u16, status: Rect) -> PanelRects {
        // Vertical: upper | bottom
        let (upper_h, bottom) = self.split_top_bottom(avail_h);

        let upper = Rect {
            x: 0,
            y: 0,
            w: self.screen_w,
            h: upper_h,
        };

        // Horizontal: tree | editor
        let mut h_constraints = Vec::new();
        if self.tree_visible {
            h_constraints.push(Constraint {
                size: Size::Fixed(self.tree_width),
                min: 10,
                max: 40,
            });
        }
        h_constraints.push(Constraint {
            size: Size::Fill,
            min: 20,
            max: u16::MAX,
        });

        let cols = upper.split(Direction::Horizontal, &h_constraints);
        let mut idx = 0;
        let tree = if self.tree_visible {
            let r = cols.get(idx).copied();
            idx += 1;
            r
        } else {
            None
        };
        let editor = cols.get(idx).copied().unwrap_or(upper);

        PanelRects {
            tree,
            editor,
            control: None,
            bottom,
            status,
            file_tabs: None,
        }
    }

    /// Split available height into top area and optional bottom panel.
    fn split_top_bottom(&self, avail_h: u16) -> (u16, Option<Rect>) {
        if !self.bottom_visible {
            return (avail_h, None);
        }
        let bot_h = (avail_h as u32 * self.bottom_height_pct as u32 / 100) as u16;
        let bot_h = bot_h.clamp(3, avail_h.saturating_sub(5));
        let top_h = avail_h.saturating_sub(bot_h);
        let bottom = Rect {
            x: 0,
            y: top_h,
            w: self.screen_w,
            h: bot_h,
        };
        (top_h, Some(bottom))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_mode_wide() {
        assert_eq!(LayoutMode::auto_select(200), LayoutMode::Wide);
    }

    #[test]
    fn auto_mode_tall_right() {
        assert_eq!(LayoutMode::auto_select(130), LayoutMode::TallRight);
    }

    #[test]
    fn auto_mode_tall_bottom() {
        assert_eq!(LayoutMode::auto_select(80), LayoutMode::TallBottom);
    }

    #[test]
    fn status_bar_last_row() {
        let mut ls = LayoutState::default();
        ls.resize(120, 40);
        let rects = ls.compute_rects();
        assert_eq!(rects.status.y, 39);
        assert_eq!(rects.status.h, 1);
    }

    #[test]
    fn hidden_tree_expands_editor() {
        let mut ls = LayoutState::default();
        ls.resize(120, 40);
        ls.tree_visible = false;
        let rects = ls.compute_rects();
        assert!(rects.tree.is_none());
        assert_eq!(rects.editor.x, 0);
    }

    #[test]
    fn hidden_bottom_expands_top() {
        let mut ls = LayoutState::default();
        ls.resize(120, 40);
        ls.bottom_visible = false;
        let rects = ls.compute_rects();
        assert!(rects.bottom.is_none());
    }

    #[test]
    fn tree_width_clamps() {
        let mut ls = LayoutState::default();
        ls.resize_tree(-100);
        assert_eq!(ls.tree_width, 10);
        ls.resize_tree(100);
        assert_eq!(ls.tree_width, 40);
    }

    #[test]
    fn cycle_layout() {
        let mut ls = LayoutState::default();
        ls.cycle_mode();
        assert_eq!(ls.mode, LayoutMode::TallRight);
        ls.cycle_mode();
        assert_eq!(ls.mode, LayoutMode::TallBottom);
        ls.cycle_mode();
        assert_eq!(ls.mode, LayoutMode::Wide);
    }

    #[test]
    fn wide_layout_has_three_cols() {
        let mut ls = LayoutState::default();
        ls.resize(200, 40);
        ls.control_visible = true;
        let rects = ls.compute_rects();
        assert!(rects.tree.is_some());
        assert!(rects.control.is_some());
    }

    #[test]
    fn tall_right_no_control() {
        let mut ls = LayoutState::default();
        ls.resize(130, 40);
        let rects = ls.compute_rects();
        assert!(rects.control.is_none());
    }

    // ── Focus management tests ──────────────────────────────────

    #[test]
    fn initial_focus_is_editor() {
        assert_eq!(PanelFocus::default(), PanelFocus::Editor);
    }

    #[test]
    fn initial_triptych_focus_is_editor() {
        assert_eq!(TriptychFocus::default(), TriptychFocus::Editor);
    }

    #[test]
    fn layout_mode_cycle_wraps() {
        assert_eq!(LayoutMode::Wide.next(), LayoutMode::TallRight);
        assert_eq!(LayoutMode::TallRight.next(), LayoutMode::TallBottom);
        assert_eq!(LayoutMode::TallBottom.next(), LayoutMode::Wide);
    }

    // ── Layout computation tests ────────────────────────────────

    #[test]
    fn wide_layout_tree_editor_control() {
        let mut ls = LayoutState::default();
        ls.resize(200, 40);
        ls.control_visible = true;
        let rects = ls.compute_rects();
        let tree = rects.tree.as_ref();
        let ctrl = rects.control.as_ref();
        assert!(tree.is_some());
        assert!(ctrl.is_some());
        // Tree is leftmost.
        assert_eq!(tree.map(|r| r.x), Some(0));
        // Editor is between tree and control.
        assert!(rects.editor.x > 0);
        // Control is rightmost.
        assert!(ctrl.map(|r| r.x) > Some(rects.editor.x));
    }

    #[test]
    fn tall_right_bottom_inside_right_col() {
        let mut ls = LayoutState::default();
        ls.resize(130, 40);
        ls.bottom_visible = true;
        let rects = ls.compute_rects();
        assert!(rects.control.is_none());
        if let Some(bot) = rects.bottom {
            // Bottom should be to the right of tree.
            if let Some(tree) = rects.tree {
                assert!(bot.x >= tree.x + tree.w);
            }
        }
    }

    #[test]
    fn tall_bottom_full_width_bottom() {
        let mut ls = LayoutState::default();
        ls.resize(80, 40);
        ls.bottom_visible = true;
        let rects = ls.compute_rects();
        assert!(rects.control.is_none());
        if let Some(bot) = rects.bottom {
            assert_eq!(bot.w, 80);
        }
    }

    #[test]
    fn hidden_tree_editor_starts_at_zero() {
        let mut ls = LayoutState::default();
        ls.resize(200, 40);
        ls.tree_visible = false;
        let rects = ls.compute_rects();
        assert!(rects.tree.is_none());
        assert_eq!(rects.editor.x, 0);
    }

    #[test]
    fn hidden_bottom_top_area_expands() {
        let mut ls = LayoutState::default();
        ls.resize(120, 40);
        ls.bottom_visible = true;
        let rects_with = ls.compute_rects();
        ls.bottom_visible = false;
        let rects_without = ls.compute_rects();
        assert!(rects_without.bottom.is_none());
        assert!(rects_without.editor.h > rects_with.editor.h);
    }

    #[test]
    fn resize_tree_clamps_min_max() {
        let mut ls = LayoutState::default();
        ls.tree_width = 20;
        ls.resize_tree(-100);
        assert_eq!(ls.tree_width, 10);
        ls.resize_tree(100);
        assert_eq!(ls.tree_width, 40);
    }

    #[test]
    fn status_bar_always_last_row() {
        for h in [10, 24, 50, 100] {
            let mut ls = LayoutState::default();
            ls.resize(120, h);
            let rects = ls.compute_rects();
            assert_eq!(rects.status.y, h - 1);
            assert_eq!(rects.status.h, 1);
        }
    }

    #[test]
    fn resize_bottom_clamps() {
        let mut ls = LayoutState::default();
        ls.resize_bottom(-100);
        assert_eq!(ls.bottom_height_pct, 10);
        ls.resize_bottom(200);
        assert_eq!(ls.bottom_height_pct, 70);
    }

    #[test]
    fn manual_mode_preserved_on_resize() {
        let mut ls = LayoutState::default();
        ls.resize(200, 40);
        ls.cycle_mode(); // Now TallRight, manual=true
        assert_eq!(ls.mode, LayoutMode::TallRight);
        ls.resize(200, 40); // Resize shouldn't change mode
        assert_eq!(ls.mode, LayoutMode::TallRight);
    }

    #[test]
    fn auto_mode_when_not_manual() {
        let mut ls = LayoutState::default();
        ls.resize(200, 40);
        assert_eq!(ls.mode, LayoutMode::Wide);
        ls.resize(80, 40);
        assert_eq!(ls.mode, LayoutMode::TallBottom);
    }

    // ── Panel visibility toggle tests ───────────────────────────

    #[test]
    fn toggle_tree_reclaims_space() {
        let mut ls = LayoutState::default();
        ls.resize(200, 40);
        ls.control_visible = true;
        let rects_with = ls.compute_rects();
        ls.tree_visible = false;
        let rects_without = ls.compute_rects();
        assert!(
            rects_without.editor.w > rects_with.editor.w,
            "editor should be wider when tree hidden"
        );
    }

    #[test]
    fn toggle_bottom_reclaims_space() {
        let mut ls = LayoutState::default();
        ls.resize(120, 40);
        ls.bottom_visible = true;
        let rects_with = ls.compute_rects();
        ls.bottom_visible = false;
        let rects_without = ls.compute_rects();
        assert!(
            rects_without.editor.h > rects_with.editor.h,
            "editor should be taller when bottom hidden"
        );
    }
}
