//! # txv-widgets
//!
//! Concrete View implementations — ready-to-use interactive TUI components.
//! Depends only on txv-core (plus `ignore` for filesystem widgets).

pub mod scroll_view;
pub mod scrollbar;
pub mod tree_view;
pub mod list_view;
pub mod input_line;
pub mod tab_bar;
pub mod status_bar;
pub mod text_area;
pub mod table;
pub mod menu;
pub mod dialog;
pub mod fuzzy_select;
pub mod overlay;
pub mod progress_bar;
pub mod split_pane;
pub mod file_tree;
pub mod file_list;

pub use dialog::Dialog;
pub use file_list::FileListData;
pub use file_tree::FileTreeData;
pub use fuzzy_select::FuzzySelect;
pub use input_line::InputLine;
pub use list_view::{ListData, ListView};
pub use menu::{Menu, MenuItem};
pub use overlay::Overlay;
pub use progress_bar::{ProgressBar, ProgressMode};
pub use scroll_view::ScrollView;
pub use scrollbar::Scrollbar;
pub use split_pane::{SplitDirection, SplitPane};
pub use status_bar::{StatusBar, StatusItem};
pub use tab_bar::{Tab, TabBar};
pub use table::{Column, Table};
pub use text_area::TextArea;
pub use tree_view::{TreeData, TreeView};
